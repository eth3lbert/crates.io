use chrono::NaiveDateTime;
use diesel::PgConnection;
use std::collections::HashMap;

use crate::github;
use crate::models::{
    Badge, Category, Crate, CrateOwnerInvitation, CreatedApiToken, Dependency, DependencyKind,
    Keyword, Owner, ReverseDependency, Team, User, Version, VersionDownload, VersionOwnerAction,
};
use crate::util::rfc3339;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct EncodableBadge {
    pub badge_type: String,
    pub attributes: HashMap<String, Option<String>>,
}

impl From<Badge> for EncodableBadge {
    fn from(badge: Badge) -> Self {
        // The serde attributes on Badge ensure it can be deserialized to EncodableBadge
        serde_json::from_value(serde_json::to_value(badge).unwrap()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableCategory {
    pub id: String,
    pub category: String,
    pub slug: String,
    pub description: String,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
    pub crates_cnt: i32,
}

impl From<Category> for EncodableCategory {
    fn from(category: Category) -> Self {
        let Category {
            crates_cnt,
            category,
            slug,
            description,
            created_at,
            ..
        } = category;
        Self {
            id: slug.clone(),
            slug,
            description,
            created_at,
            crates_cnt,
            category: category.rsplit("::").collect::<Vec<_>>()[0].to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableCategoryWithSubcategories {
    pub id: String,
    pub category: String,
    pub slug: String,
    pub description: String,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
    pub crates_cnt: i32,
    pub subcategories: Vec<EncodableCategory>,
    pub parent_categories: Vec<EncodableCategory>,
}

/// The serialization format for the `CrateOwnerInvitation` model.
#[derive(Deserialize, Serialize, Debug)]
pub struct EncodableCrateOwnerInvitation {
    pub invited_by_username: String,
    pub crate_name: String,
    pub crate_id: i32,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
}

impl EncodableCrateOwnerInvitation {
    pub fn from(invitation: CrateOwnerInvitation, conn: &PgConnection) -> Self {
        Self {
            invited_by_username: invitation.invited_by_username(conn),
            crate_name: invitation.crate_name(conn),
            crate_id: invitation.crate_id,
            created_at: invitation.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct InvitationResponse {
    pub crate_id: i32,
    pub accepted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableDependency {
    pub id: i32,
    pub version_id: i32,
    pub crate_id: String,
    pub req: String,
    pub optional: bool,
    pub default_features: bool,
    pub features: Vec<String>,
    pub target: Option<String>,
    pub kind: DependencyKind,
    pub downloads: i32,
}

impl EncodableDependency {
    pub fn from_dep(dependency: Dependency, crate_name: &str) -> Self {
        Self::encode(dependency, crate_name, None)
    }

    pub fn from_reverse_dep(rev_dep: ReverseDependency, crate_name: &str) -> Self {
        let dependency = rev_dep.dependency;
        Self::encode(dependency, crate_name, Some(rev_dep.crate_downloads))
    }

    // `downloads` need only be specified when generating a reverse dependency
    fn encode(dependency: Dependency, crate_name: &str, downloads: Option<i32>) -> Self {
        Self {
            id: dependency.id,
            version_id: dependency.version_id,
            crate_id: crate_name.into(),
            req: dependency.req,
            optional: dependency.optional,
            default_features: dependency.default_features,
            features: dependency.features,
            target: dependency.target,
            kind: dependency.kind,
            downloads: downloads.unwrap_or(0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableVersionDownload {
    pub version: i32,
    pub downloads: i32,
    pub date: String,
}

impl From<VersionDownload> for EncodableVersionDownload {
    fn from(download: VersionDownload) -> Self {
        Self {
            version: download.version_id,
            downloads: download.downloads,
            date: download.date.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableKeyword {
    pub id: String,
    pub keyword: String,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
    pub crates_cnt: i32,
}

impl From<Keyword> for EncodableKeyword {
    fn from(keyword: Keyword) -> Self {
        let Keyword {
            crates_cnt,
            keyword,
            created_at,
            ..
        } = keyword;
        Self {
            id: keyword.clone(),
            created_at,
            crates_cnt,
            keyword,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableCrate {
    pub id: String,
    pub name: String,
    #[serde(with = "rfc3339")]
    pub updated_at: NaiveDateTime,
    pub versions: Option<Vec<i32>>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub badges: Option<Vec<EncodableBadge>>,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
    // NOTE: Used by shields.io, altering `downloads` requires a PR with shields.io
    pub downloads: i32,
    pub recent_downloads: Option<i64>,
    // NOTE: Used by shields.io, altering `max_version` requires a PR with shields.io
    pub max_version: String,
    pub newest_version: String, // Most recently updated version, which may not be max
    pub max_stable_version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub repository: Option<String>,
    pub links: EncodableCrateLinks,
    pub exact_match: bool,
}

impl EncodableCrate {
    #[allow(clippy::too_many_arguments)]
    pub fn from(
        krate: Crate,
        top_versions: &TopVersions,
        versions: Option<Vec<i32>>,
        keywords: Option<&[Keyword]>,
        categories: Option<&[Category]>,
        badges: Option<Vec<Badge>>,
        exact_match: bool,
        recent_downloads: Option<i64>,
    ) -> Self {
        krate.encodable(
            top_versions,
            versions,
            keywords,
            categories,
            badges,
            exact_match,
            recent_downloads,
        )
    }

    pub fn from_minimal(
        krate: Crate,
        top_versions: &TopVersions,
        badges: Option<Vec<Badge>>,
        exact_match: bool,
        recent_downloads: Option<i64>,
    ) -> Self {
        Self::from(
            krate,
            top_versions,
            None,
            None,
            None,
            badges,
            exact_match,
            recent_downloads,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableCrateLinks {
    pub version_downloads: String,
    pub versions: Option<String>,
    pub owners: Option<String>,
    pub owner_team: Option<String>,
    pub owner_user: Option<String>,
    pub reverse_dependencies: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableOwner {
    pub id: i32,
    pub login: String,
    pub kind: String,
    pub url: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
}

impl From<Owner> for EncodableOwner {
    fn from(owner: Owner) -> Self {
        match owner {
            Owner::User(User {
                id,
                name,
                gh_login,
                gh_avatar,
                ..
            }) => {
                let url = format!("https://github.com/{}", gh_login);
                Self {
                    id,
                    login: gh_login,
                    avatar: gh_avatar,
                    url: Some(url),
                    name,
                    kind: String::from("user"),
                }
            }
            Owner::Team(Team {
                id,
                name,
                login,
                avatar,
                ..
            }) => {
                let url = github::team_url(&login);
                Self {
                    id,
                    login,
                    url: Some(url),
                    avatar,
                    name,
                    kind: String::from("team"),
                }
            }
        }
    }
}

#[derive(Serialize, Debug)]
pub struct EncodableTeam {
    pub id: i32,
    pub login: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub url: Option<String>,
}

impl From<Team> for EncodableTeam {
    fn from(team: Team) -> Self {
        let Team {
            id,
            name,
            login,
            avatar,
            ..
        } = team;
        let url = github::team_url(&login);

        EncodableTeam {
            id,
            login,
            name,
            avatar,
            url: Some(url),
        }
    }
}

/// The serialization format for the `ApiToken` model with its token value.
/// This should only be used when initially creating a new token to minimize
/// the chance of token leaks.
#[derive(Deserialize, Serialize, Debug)]
pub struct EncodableApiTokenWithToken {
    pub id: i32,
    pub name: String,
    pub token: String,
    pub revoked: bool,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
    #[serde(with = "rfc3339::option")]
    pub last_used_at: Option<NaiveDateTime>,
}

impl From<CreatedApiToken> for EncodableApiTokenWithToken {
    fn from(token: CreatedApiToken) -> Self {
        EncodableApiTokenWithToken {
            id: token.model.id,
            name: token.model.name,
            token: token.plaintext,
            revoked: token.model.revoked,
            created_at: token.model.created_at,
            last_used_at: token.model.last_used_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OwnedCrate {
    pub id: i32,
    pub name: String,
    pub email_notifications: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableMe {
    pub user: EncodablePrivateUser,
    pub owned_crates: Vec<OwnedCrate>,
}

/// The serialization format for the `User` model.
/// Same as public user, except for addition of
/// email field
#[derive(Deserialize, Serialize, Debug)]
pub struct EncodablePrivateUser {
    pub id: i32,
    pub login: String,
    pub email_verified: bool,
    pub email_verification_sent: bool,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub url: Option<String>,
}

impl EncodablePrivateUser {
    /// Converts this `User` model into an `EncodablePrivateUser` for JSON serialization.
    pub fn from(
        user: User,
        email: Option<String>,
        email_verified: bool,
        email_verification_sent: bool,
    ) -> Self {
        let User {
            id,
            name,
            gh_login,
            gh_avatar,
            ..
        } = user;
        let url = format!("https://github.com/{}", gh_login);

        EncodablePrivateUser {
            id,
            email,
            email_verified,
            email_verification_sent,
            avatar: gh_avatar,
            login: gh_login,
            name,
            url: Some(url),
        }
    }
}

/// The serialization format for the `User` model.
/// Same as private user, except no email field
#[derive(Deserialize, Serialize, Debug)]
pub struct EncodablePublicUser {
    pub id: i32,
    pub login: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub url: Option<String>,
}

/// Converts a `User` model into an `EncodablePublicUser` for JSON serialization.
impl From<User> for EncodablePublicUser {
    fn from(user: User) -> Self {
        let User {
            id,
            name,
            gh_login,
            gh_avatar,
            ..
        } = user;
        let url = format!("https://github.com/{}", gh_login);
        EncodablePublicUser {
            id,
            avatar: gh_avatar,
            login: gh_login,
            name,
            url: Some(url),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EncodableAuditAction {
    pub action: String,
    pub user: EncodablePublicUser,
    #[serde(with = "rfc3339")]
    pub time: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableVersion {
    pub id: i32,
    #[serde(rename = "crate")]
    pub krate: String,
    pub num: String,
    pub dl_path: String,
    pub readme_path: String,
    #[serde(with = "rfc3339")]
    pub updated_at: NaiveDateTime,
    #[serde(with = "rfc3339")]
    pub created_at: NaiveDateTime,
    // NOTE: Used by shields.io, altering `downloads` requires a PR with shields.io
    pub downloads: i32,
    pub features: serde_json::Value,
    pub yanked: bool,
    // NOTE: Used by shields.io, altering `license` requires a PR with shields.io
    pub license: Option<String>,
    pub links: EncodableVersionLinks,
    pub crate_size: Option<i32>,
    pub published_by: Option<EncodablePublicUser>,
    pub audit_actions: Vec<EncodableAuditAction>,
}

impl EncodableVersion {
    pub fn from(
        version: Version,
        crate_name: &str,
        published_by: Option<User>,
        audit_actions: Vec<(VersionOwnerAction, User)>,
    ) -> Self {
        let Version {
            id,
            num,
            updated_at,
            created_at,
            downloads,
            features,
            yanked,
            license,
            crate_size,
            ..
        } = version;

        let num = num.to_string();

        Self {
            dl_path: format!("/api/v1/crates/{}/{}/download", crate_name, num),
            readme_path: format!("/api/v1/crates/{}/{}/readme", crate_name, num),
            num: num.clone(),
            id,
            krate: crate_name.to_string(),
            updated_at,
            created_at,
            downloads,
            features,
            yanked,
            license,
            links: EncodableVersionLinks {
                dependencies: format!("/api/v1/crates/{}/{}/dependencies", crate_name, num),
                version_downloads: format!("/api/v1/crates/{}/{}/downloads", crate_name, num),
                authors: format!("/api/v1/crates/{}/{}/authors", crate_name, num),
            },
            crate_size,
            published_by: published_by.map(User::into),
            audit_actions: audit_actions
                .into_iter()
                .map(|(audit_action, user)| EncodableAuditAction {
                    action: audit_action.action.into(),
                    user: user.into(),
                    time: audit_action.time,
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableVersionLinks {
    pub dependencies: String,
    pub version_downloads: String,
    pub authors: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoodCrate {
    #[serde(rename = "crate")]
    pub krate: EncodableCrate,
    pub warnings: PublishWarnings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublishWarnings {
    pub invalid_categories: Vec<String>,
    pub invalid_badges: Vec<String>,
    pub other: Vec<String>,
}

pub mod krate_publish;
pub use self::krate_publish::{EncodableCrateDependency, EncodableCrateUpload};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn category_dates_serializes_to_rfc3339() {
        let cat = EncodableCategory {
            id: "".to_string(),
            category: "".to_string(),
            slug: "".to_string(),
            description: "".to_string(),
            crates_cnt: 1,
            created_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 11),
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert_some!(json
            .as_str()
            .find(r#""created_at":"2017-01-06T14:23:11+00:00""#));
    }

    #[test]
    fn category_with_sub_dates_serializes_to_rfc3339() {
        let cat = EncodableCategoryWithSubcategories {
            id: "".to_string(),
            category: "".to_string(),
            slug: "".to_string(),
            description: "".to_string(),
            crates_cnt: 1,
            created_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 11),
            subcategories: vec![],
            parent_categories: vec![],
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert_some!(json
            .as_str()
            .find(r#""created_at":"2017-01-06T14:23:11+00:00""#));
    }

    #[test]
    fn keyword_serializes_to_rfc3339() {
        let key = EncodableKeyword {
            id: "".to_string(),
            keyword: "".to_string(),
            created_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 11),
            crates_cnt: 0,
        };
        let json = serde_json::to_string(&key).unwrap();
        assert_some!(json
            .as_str()
            .find(r#""created_at":"2017-01-06T14:23:11+00:00""#));
    }

    #[test]
    fn version_serializes_to_rfc3339() {
        let ver = EncodableVersion {
            id: 1,
            krate: "".to_string(),
            num: "".to_string(),
            dl_path: "".to_string(),
            readme_path: "".to_string(),
            updated_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 11),
            created_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 12),
            downloads: 0,
            features: serde_json::from_str("{}").unwrap(),
            yanked: false,
            license: None,
            links: EncodableVersionLinks {
                dependencies: "".to_string(),
                version_downloads: "".to_string(),
                authors: "".to_string(),
            },
            crate_size: Some(1234),
            published_by: None,
            audit_actions: vec![EncodableAuditAction {
                action: "publish".to_string(),
                user: EncodablePublicUser {
                    id: 0,
                    login: String::new(),
                    name: None,
                    avatar: None,
                    url: None,
                },
                time: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 12),
            }],
        };
        let json = serde_json::to_string(&ver).unwrap();
        assert_some!(json
            .as_str()
            .find(r#""updated_at":"2017-01-06T14:23:11+00:00""#));
        assert_some!(json
            .as_str()
            .find(r#""created_at":"2017-01-06T14:23:12+00:00""#));
        assert_some!(json.as_str().find(r#""time":"2017-01-06T14:23:12+00:00""#));
    }

    #[test]
    fn crate_serializes_to_rfc3399() {
        let crt = EncodableCrate {
            id: "".to_string(),
            name: "".to_string(),
            updated_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 11),
            versions: None,
            keywords: None,
            categories: None,
            badges: None,
            created_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 12),
            downloads: 0,
            recent_downloads: None,
            max_version: "".to_string(),
            newest_version: "".to_string(),
            max_stable_version: None,
            description: None,
            homepage: None,
            documentation: None,
            repository: None,
            links: EncodableCrateLinks {
                version_downloads: "".to_string(),
                versions: None,
                owners: None,
                owner_team: None,
                owner_user: None,
                reverse_dependencies: "".to_string(),
            },
            exact_match: false,
        };
        let json = serde_json::to_string(&crt).unwrap();
        assert_some!(json
            .as_str()
            .find(r#""updated_at":"2017-01-06T14:23:11+00:00""#));
        assert_some!(json
            .as_str()
            .find(r#""created_at":"2017-01-06T14:23:12+00:00""#));
    }

    #[test]
    fn crate_owner_invitation_serializes_to_rfc3339() {
        let inv = EncodableCrateOwnerInvitation {
            invited_by_username: "".to_string(),
            crate_name: "".to_string(),
            crate_id: 123,
            created_at: NaiveDate::from_ymd(2017, 1, 6).and_hms(14, 23, 11),
        };
        let json = serde_json::to_string(&inv).unwrap();
        assert_some!(json
            .as_str()
            .find(r#""created_at":"2017-01-06T14:23:11+00:00""#));
    }
}

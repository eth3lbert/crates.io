WITH filtered_default_versions as (
    SELECT default_versions.*
    FROM default_versions
    WHERE version_id IN (SELECT version_id FROM dependencies WHERE crate_id = $1)
        AND NOT EXISTS (
            SELECT 1
            FROM versions
            WHERE id = version_id and yanked
    )
)
SELECT
    dependencies.*,
    crate_downloads.downloads as crate_downloads,
    crates.name as crate_name,
    (SELECT COUNT(*) from filtered_default_versions) as total
FROM filtered_default_versions
INNER JOIN crates
    ON crates.id = filtered_default_versions.crate_id
INNER JOIN crate_downloads using (crate_id)
-- Multiple dependencies can exist, we only want first one
CROSS JOIN LATERAL (
    SELECT dependencies.*
    FROM dependencies
    WHERE dependencies.crate_id = $1 AND dependencies.version_id = filtered_default_versions.version_id
    ORDER BY id ASC
    LIMIT 1
) dependencies
ORDER BY
    crate_downloads DESC,
    crate_name ASC
OFFSET $2
LIMIT $3

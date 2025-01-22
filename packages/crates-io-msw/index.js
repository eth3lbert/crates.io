import apiTokenHandlers from './handlers/api-tokens.js';
import categoryHandlers from './handlers/categories.js';
import docsRsHandlers from './handlers/docs-rs.js';
import inviteHandlers from './handlers/invites.js';
import keywordHandlers from './handlers/keywords.js';
import metadataHandlers from './handlers/metadata.js';
import sessionHandlers from './handlers/sessions.js';
import teamHandlers from './handlers/teams.js';
import userHandlers from './handlers/users.js';
import apiToken from './models/api-token.js';
import category from './models/category.js';
import crateOwnerInvitation from './models/crate-owner-invitation.js';
import crateOwnership from './models/crate-ownership.js';
import crate from './models/crate.js';
import dependency from './models/dependency.js';
import keyword from './models/keyword.js';
import mswSession from './models/msw-session.js';
import team from './models/team.js';
import user from './models/user.js';
import versionDownload from './models/version-download.js';
import version from './models/version.js';
import { factory } from './utils/factory.js';

export const handlers = [
  ...apiTokenHandlers,
  ...categoryHandlers,
  ...docsRsHandlers,
  ...inviteHandlers,
  ...keywordHandlers,
  ...metadataHandlers,
  ...sessionHandlers,
  ...teamHandlers,
  ...userHandlers,
];

export const db = factory({
  apiToken,
  category,
  crateOwnerInvitation,
  crateOwnership,
  crate,
  dependency,
  keyword,
  mswSession,
  team,
  user,
  versionDownload,
  version,
});

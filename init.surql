DEFINE TABLE player SCHEMAFULL;
DEFINE FIELD id ON player TYPE string ASSERT string::is::email($value);
DEFINE FIELD phone ON player TYPE string VALUE crypto::argon2::generate($value);
DEFINE INDEX emailu ON player FIELDS id UNIQUE;
DEFINE INDEX phoneu ON player FIELDS phone UNIQUE;

DEFINE TABLE submission SCHEMAFULL;
DEFINE FIELD player ON submission TYPE record<player>;
DEFINE FIELD language ON submission TYPE string ASSERT $value IN ["rust", "javascript", "cpp", "python", "java"];
DEFINE FIELD code ON submission TYPE string;
DEFINE FIELD status ON submission FLEXIBLE TYPE object;
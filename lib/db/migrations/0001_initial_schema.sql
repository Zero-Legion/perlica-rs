CREATE TABLE IF NOT EXISTS beyond_players (
    uid                  TEXT    NOT NULL PRIMARY KEY,
    -- WorldState
    role_level           INTEGER NOT NULL DEFAULT 1,
    role_exp             INTEGER NOT NULL DEFAULT 0,
    last_scene           TEXT    NOT NULL DEFAULT 'map01_lv001',
    pos_x                REAL    NOT NULL DEFAULT 469.0,
    pos_y                REAL    NOT NULL DEFAULT 107.11,
    pos_z                REAL    NOT NULL DEFAULT 217.83,
    rot_x                REAL    NOT NULL DEFAULT 0.0,
    rot_y                REAL    NOT NULL DEFAULT 60.0,
    rot_z                REAL    NOT NULL DEFAULT 0.0,
    -- CharBag meta
    curr_team_index      INTEGER NOT NULL DEFAULT 0,
    -- MissionManager
    track_mission_id     TEXT    NOT NULL DEFAULT '',
    -- RevivalMode (0=Default, 1=RepatriatePoint, 2=CheckPoint)
    revival_mode         INTEGER NOT NULL DEFAULT 0,
    -- CheckpointInfo (NULL = no checkpoint set)
    checkpoint_scene     TEXT,
    checkpoint_x         REAL,
    checkpoint_y         REAL,
    checkpoint_z         REAL,
    -- Instance-ID counters for instanced depots
    weapon_next_inst_id  INTEGER NOT NULL DEFAULT 1,
    gem_next_inst_id     INTEGER NOT NULL DEFAULT 1,
    equip_next_inst_id   INTEGER NOT NULL DEFAULT 1,
    -- MailManager counter
    mail_next_id         INTEGER NOT NULL DEFAULT 1,
    -- Housekeeping
    updated_at           INTEGER NOT NULL DEFAULT 0
);

-- Characters
CREATE TABLE IF NOT EXISTS beyond_chars (
    uid          TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    char_index   INTEGER NOT NULL,   -- position in CharBag.chars Vec
    template_id  TEXT    NOT NULL,
    level        INTEGER NOT NULL DEFAULT 1,
    exp          INTEGER NOT NULL DEFAULT 0,
    break_stage  INTEGER NOT NULL DEFAULT 0,
    is_dead      INTEGER NOT NULL DEFAULT 0,   -- bool
    hp           REAL    NOT NULL DEFAULT 0.0,
    ultimate_sp  REAL    NOT NULL DEFAULT 0.0,
    own_time     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, char_index)
);

-- Per-character skill levels.  skill_id is the game template ID (e.g.
-- "char_aknight_normal_skill_1").  Skill levels are stored as a flat table
-- rather than a map column so queries and partial updates remain typed.
CREATE TABLE IF NOT EXISTS beyond_char_skills (
    uid          TEXT    NOT NULL,
    char_index   INTEGER NOT NULL,
    skill_id     TEXT    NOT NULL,
    skill_level  INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (uid, char_index, skill_id),
    FOREIGN KEY (uid, char_index)
        REFERENCES beyond_chars(uid, char_index) ON DELETE CASCADE
);

-- Teams
CREATE TABLE IF NOT EXISTS beyond_teams (
    uid               TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    team_index        INTEGER NOT NULL,
    team_name         TEXT    NOT NULL DEFAULT '',
    leader_char_index INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, team_index)
);

-- Four slots per team.  char_index IS NULL for empty slots.
CREATE TABLE IF NOT EXISTS beyond_team_slots (
    uid         TEXT    NOT NULL,
    team_index  INTEGER NOT NULL,
    slot_index  INTEGER NOT NULL CHECK (slot_index BETWEEN 0 AND 3),
    char_index  INTEGER,   -- NULL = TeamSlot::Empty
    PRIMARY KEY (uid, team_index, slot_index),
    FOREIGN KEY (uid, team_index)
        REFERENCES beyond_teams(uid, team_index) ON DELETE CASCADE
);

-- Weapon depot
CREATE TABLE IF NOT EXISTS beyond_weapons (
    uid              TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    inst_id          INTEGER NOT NULL,   -- WeaponInstId
    template_id      TEXT    NOT NULL,
    exp              INTEGER NOT NULL DEFAULT 0,
    weapon_lv        INTEGER NOT NULL DEFAULT 1,
    refine_lv        INTEGER NOT NULL DEFAULT 0,
    breakthrough_lv  INTEGER NOT NULL DEFAULT 0,
    equip_char_id    INTEGER NOT NULL DEFAULT 0,   -- 0 = unequipped
    attach_gem_id    INTEGER NOT NULL DEFAULT 0,   -- 0 = no gem
    is_lock          INTEGER NOT NULL DEFAULT 0,   -- bool
    is_new           INTEGER NOT NULL DEFAULT 1,   -- bool
    own_time         INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, inst_id)
);

-- Gem depot
-- craft_slot is a CraftShowingType discriminant (INTEGER).
CREATE TABLE IF NOT EXISTS beyond_gems (
    uid               TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    inst_id           INTEGER NOT NULL,
    template_id       TEXT    NOT NULL,
    craft_slot        INTEGER NOT NULL DEFAULT 0,
    attach_weapon_id  INTEGER NOT NULL DEFAULT 0,  -- 0 = not socketed
    is_lock           INTEGER NOT NULL DEFAULT 0,
    is_new            INTEGER NOT NULL DEFAULT 1,
    own_time          INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, inst_id)
);

-- Equip depot
-- slot is a CraftShowingType discriminant (EquipHead=7, EquipBody=8, EquipRing=9).
CREATE TABLE IF NOT EXISTS beyond_equips (
    uid            TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    inst_id        INTEGER NOT NULL,
    template_id    TEXT    NOT NULL,
    slot           INTEGER NOT NULL DEFAULT 0,
    equip_char_id  INTEGER NOT NULL DEFAULT 0,
    is_lock        INTEGER NOT NULL DEFAULT 0,
    is_new         INTEGER NOT NULL DEFAULT 1,
    own_time       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, inst_id)
);

-- Equip stat modifiers: one row per (piece * attribute).
CREATE TABLE IF NOT EXISTS beyond_equip_attrs (
    uid             TEXT    NOT NULL,
    inst_id         INTEGER NOT NULL,
    attr_index      INTEGER NOT NULL,  -- ordinal within the attrs Vec
    attr_type       INTEGER NOT NULL,
    modifier_type   INTEGER NOT NULL,
    modifier_value  REAL    NOT NULL,
    PRIMARY KEY (uid, inst_id, attr_index),
    FOREIGN KEY (uid, inst_id)
        REFERENCES beyond_equips(uid, inst_id) ON DELETE CASCADE
);

-- Stackable item depots
CREATE TABLE IF NOT EXISTS beyond_stackable_items (
    uid          TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    depot_type   INTEGER NOT NULL,
    template_id  TEXT    NOT NULL,
    count        INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, depot_type, template_id)
);

--  Boolean flag sets (bitsets)
-- bitset_type is a BitsetType discriminant (0–19).
-- bit_value is the game-internal ID that was set.
CREATE TABLE IF NOT EXISTS beyond_bitsets (
    uid          TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    bitset_type  INTEGER NOT NULL,
    bit_value    INTEGER NOT NULL,
    PRIMARY KEY (uid, bitset_type, bit_value)
);

--  Mission progress
CREATE TABLE IF NOT EXISTS beyond_missions (
    uid            TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    mission_id     TEXT    NOT NULL,
    mission_state  INTEGER NOT NULL DEFAULT 0,
    succeed_id     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, mission_id)
);

-- Active quests.
CREATE TABLE IF NOT EXISTS beyond_quests (
    uid          TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    quest_id     TEXT    NOT NULL,
    quest_state  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, quest_id)
);

-- Per-quest objective progress.
-- objective_value stores values[condition_id], which is always the single entry
-- in the map (key == condition_id).  extra_details is always empty in practice
-- and is reconstructed as such on load.
CREATE TABLE IF NOT EXISTS beyond_quest_objectives (
    uid              TEXT    NOT NULL,
    quest_id         TEXT    NOT NULL,
    condition_id     TEXT    NOT NULL,
    is_complete      INTEGER NOT NULL DEFAULT 0,
    objective_value  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, quest_id, condition_id),
    FOREIGN KEY (uid, quest_id)
        REFERENCES beyond_quests(uid, quest_id) ON DELETE CASCADE
);

-- Guide completions
CREATE TABLE IF NOT EXISTS beyond_guide_completions (
    uid              TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    completion_type  INTEGER NOT NULL,
    guide_id         TEXT    NOT NULL,
    PRIMARY KEY (uid, completion_type, guide_id)
);

--  Mail
CREATE TABLE IF NOT EXISTS beyond_mails (
    uid                TEXT    NOT NULL REFERENCES beyond_players(uid) ON DELETE CASCADE,
    mail_id            INTEGER NOT NULL,
    mail_type          INTEGER NOT NULL DEFAULT 0,
    is_read            INTEGER NOT NULL DEFAULT 0,
    is_attachment_got  INTEGER NOT NULL DEFAULT 0,
    send_time          INTEGER NOT NULL DEFAULT 0,
    expire_time        INTEGER NOT NULL DEFAULT -1,
    template_id        TEXT    NOT NULL DEFAULT '',
    title              TEXT    NOT NULL DEFAULT '',
    content            TEXT    NOT NULL DEFAULT '',
    sender_name        TEXT    NOT NULL DEFAULT '',
    sender_icon        TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (uid, mail_id)
);

-- Mail attachment items.
CREATE TABLE IF NOT EXISTS beyond_mail_attachments (
    uid              TEXT    NOT NULL,
    mail_id          INTEGER NOT NULL,
    item_index       INTEGER NOT NULL,  -- ordinal within the items Vec
    item_template_id TEXT    NOT NULL,
    item_count       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (uid, mail_id, item_index),
    FOREIGN KEY (uid, mail_id)
        REFERENCES beyond_mails(uid, mail_id) ON DELETE CASCADE
);

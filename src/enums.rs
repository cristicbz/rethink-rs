pub mod term {
    pub const DATUM: u32 = 1;
    pub const MAKE_ARRAY: u32 = 2;
    pub const MAKE_OBJ: u32 = 3;
    pub const VAR: u32 = 10;
    pub const JAVASCRIPT: u32 = 11;
    pub const UUID: u32 = 169;
    pub const HTTP: u32 = 153;
    pub const ERROR: u32 = 12;
    pub const IMPLICIT_VAR: u32 = 13;
    pub const DB: u32 = 14;
    pub const TABLE: u32 = 15;
    pub const GET: u32 = 16;
    pub const GET_ALL: u32 = 78;
    pub const EQ: u32 = 17;
    pub const NE: u32 = 18;
    pub const LT: u32 = 19;
    pub const LE: u32 = 20;
    pub const GT: u32 = 21;
    pub const GE: u32 = 22;
    pub const NOT: u32 = 23;
    pub const ADD: u32 = 24;
    pub const SUB: u32 = 25;
    pub const MUL: u32 = 26;
    pub const DIV: u32 = 27;
    pub const MOD: u32 = 28;
    pub const FLOOR: u32 = 183;
    pub const CEIL: u32 = 184;
    pub const ROUND: u32 = 185;
    pub const APPEND: u32 = 29;
    pub const PREPEND: u32 = 80;
    pub const DIFFERENCE: u32 = 95;
    pub const SET_INSERT: u32 = 88;
    pub const SET_INTERSECTION: u32 = 89;
    pub const SET_UNION: u32 = 90;
    pub const SET_DIFFERENCE: u32 = 91;
    pub const SLICE: u32 = 30;
    pub const SKIP: u32 = 70;
    pub const LIMIT: u32 = 71;
    pub const OFFSETS_OF: u32 = 87;
    pub const CONTAINS: u32 = 93;
    pub const GET_FIELD: u32 = 31;
    pub const KEYS: u32 = 94;
    pub const VALUES: u32 = 186;
    pub const OBJECT: u32 = 143;
    pub const HAS_FIELDS: u32 = 32;
    pub const WITH_FIELDS: u32 = 96;
    pub const PLUCK: u32 = 33;
    pub const WITHOUT: u32 = 34;
    pub const MERGE: u32 = 35;
    pub const BETWEEN_DEPRECATED: u32 = 36;
    pub const BETWEEN: u32 = 182;
    pub const REDUCE: u32 = 37;
    pub const MAP: u32 = 38;
    pub const FOLD: u32 = 187;
    pub const FILTER: u32 = 39;
    pub const CONCAT_MAP: u32 = 40;
    pub const ORDER_BY: u32 = 41;
    pub const DISTINCT: u32 = 42;
    pub const COUNT: u32 = 43;
    pub const IS_EMPTY: u32 = 86;
    pub const UNION: u32 = 44;
    pub const NTH: u32 = 45;
    pub const BRACKET: u32 = 170;
    pub const INNER_JOIN: u32 = 48;
    pub const OUTER_JOIN: u32 = 49;
    pub const EQ_JOIN: u32 = 50;
    pub const ZIP: u32 = 72;
    pub const RANGE: u32 = 173;
    pub const INSERT_AT: u32 = 82;
    pub const DELETE_AT: u32 = 83;
    pub const CHANGE_AT: u32 = 84;
    pub const SPLICE_AT: u32 = 85;
    pub const COERCE_TO: u32 = 51;
    pub const TYPE_OF: u32 = 52;
    pub const UPDATE: u32 = 53;
    pub const DELETE: u32 = 54;
    pub const REPLACE: u32 = 55;
    pub const INSERT: u32 = 56;
    pub const DB_CREATE: u32 = 57;
    pub const DB_DROP: u32 = 58;
    pub const DB_LIST: u32 = 59;
    pub const TABLE_CREATE: u32 = 60;
    pub const TABLE_DROP: u32 = 61;
    pub const TABLE_LIST: u32 = 62;
    pub const CONFIG: u32 = 174;
    pub const STATUS: u32 = 175;
    pub const WAIT: u32 = 177;
    pub const RECONFIGURE: u32 = 176;
    pub const REBALANCE: u32 = 179;
    pub const SYNC: u32 = 138;
    pub const GRANT: u32 = 188;
    pub const INDEX_CREATE: u32 = 75;
    pub const INDEX_DROP: u32 = 76;
    pub const INDEX_LIST: u32 = 77;
    pub const INDEX_STATUS: u32 = 139;
    pub const INDEX_WAIT: u32 = 140;
    pub const INDEX_RENAME: u32 = 156;
    pub const FUNCALL: u32 = 64;
    pub const BRANCH: u32 = 65;
    pub const OR: u32 = 66;
    pub const AND: u32 = 67;
    pub const FOR_EACH: u32 = 68;
    pub const FUNC: u32 = 69;
    pub const ASC: u32 = 73;
    pub const DESC: u32 = 74;
    pub const INFO: u32 = 79;
    pub const MATCH: u32 = 97;
    pub const UPCASE: u32 = 141;
    pub const DOWNCASE: u32 = 142;
    pub const SAMPLE: u32 = 81;
    pub const DEFAULT: u32 = 92;
    pub const JSON: u32 = 98;
    pub const TO_JSON_STRING: u32 = 172;
    pub const ISO8601: u32 = 99;
    pub const TO_ISO8601: u32 = 100;
    pub const EPOCH_TIME: u32 = 101;
    pub const TO_EPOCH_TIME: u32 = 102;
    pub const NOW: u32 = 103;
    pub const IN_TIMEZONE: u32 = 104;
    pub const DURING: u32 = 105;
    pub const DATE: u32 = 106;
    pub const TIME_OF_DAY: u32 = 126;
    pub const TIMEZONE: u32 = 127;
    pub const YEAR: u32 = 128;
    pub const MONTH: u32 = 129;
    pub const DAY: u32 = 130;
    pub const DAY_OF_WEEK: u32 = 131;
    pub const DAY_OF_YEAR: u32 = 132;
    pub const HOURS: u32 = 133;
    pub const MINUTES: u32 = 134;
    pub const SECONDS: u32 = 135;
    pub const TIME: u32 = 136;
    pub const MONDAY: u32 = 107;
    pub const TUESDAY: u32 = 108;
    pub const WEDNESDAY: u32 = 109;
    pub const THURSDAY: u32 = 110;
    pub const FRIDAY: u32 = 111;
    pub const SATURDAY: u32 = 112;
    pub const SUNDAY: u32 = 113;
    pub const JANUARY: u32 = 114;
    pub const FEBRUARY: u32 = 115;
    pub const MARCH: u32 = 116;
    pub const APRIL: u32 = 117;
    pub const MAY: u32 = 118;
    pub const JUNE: u32 = 119;
    pub const JULY: u32 = 120;
    pub const AUGUST: u32 = 121;
    pub const SEPTEMBER: u32 = 122;
    pub const OCTOBER: u32 = 123;
    pub const NOVEMBER: u32 = 124;
    pub const DECEMBER: u32 = 125;
    pub const LITERAL: u32 = 137;
    pub const GROUP: u32 = 144;
    pub const SUM: u32 = 145;
    pub const AVG: u32 = 146;
    pub const MIN: u32 = 147;
    pub const MAX: u32 = 148;
    pub const SPLIT: u32 = 149;
    pub const UNGROUP: u32 = 150;
    pub const RANDOM: u32 = 151;
    pub const CHANGES: u32 = 152;
    pub const ARGS: u32 = 154;
    pub const BINARY: u32 = 155;
    pub const GEOJSON: u32 = 157;
    pub const TO_GEOJSON: u32 = 158;
    pub const POINT: u32 = 159;
    pub const LINE: u32 = 160;
    pub const POLYGON: u32 = 161;
    pub const DISTANCE: u32 = 162;
    pub const INTERSECTS: u32 = 163;
    pub const INCLUDES: u32 = 164;
    pub const CIRCLE: u32 = 165;
    pub const GET_INTERSECTING: u32 = 166;
    pub const FILL: u32 = 167;
    pub const GET_NEAREST: u32 = 168;
    pub const POLYGON_SUB: u32 = 171;
    pub const MINVAL: u32 = 180;
    pub const MAXVAL: u32 = 181;
}
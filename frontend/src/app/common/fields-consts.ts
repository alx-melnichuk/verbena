// List of constants used for input fields.

export const CN_NICKNAME = {
    "minLength": 3,
    "maxLength": 64,
    "pattern": "^[a-zA-Z]+[\\w]+$",
};

// https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address
// What is the maximum length of a valid email address? 
// Answer: An email address must not exceed 254 characters.
export const CN_EMAIL = {
    "minLength": 5,
    "maxLength": 254,
};

export const CN_PASSWORD = {
    "minLength": 6,
    "maxLength": 64,
    "pattern": "^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)[A-Za-z\\d\\W_]{6,}$",
};

export const CN_DESCRIPT = {
    "minLength": 2,
    "maxLength": 2048, // 2*1024
    "numRows": 4,
};

export const CN_TITLE = {
    "minLength": 2,
    "maxLength": 255,
};

export const CN_TAG = {
    "minLength": 2,
    "maxLength": 32, // 255,
    "minAmount": 1,
    "maxAmount": 4,
};

// ** Route **
export const R_ROOT = "ind";

// ** Page: About **
export const R_ABOUT = R_ROOT + "/" + "about"; // "ind/about"
export const ROUTE_ABOUT = "/" + R_ABOUT;      // "/ind/about"

// ** Page: Login **
export const R_LOGIN = R_ROOT + "/" + "login"; // "ind/login"
export const ROUTE_LOGIN = "/" + R_LOGIN;      // "/ind/login"

// ** Page: Signup **
export const R_SIGNUP = R_ROOT + "/" + "signup"; // "ind/signup"
export const ROUTE_SIGNUP = "/" + R_SIGNUP;      // "/ind/signup"

// ** Page: Forgot-password **
export const R_FORGOT_PASSWORD = R_ROOT + "/" + "forgot-password"; // "ind/forgot-password"
export const ROUTE_FORGOT_PASSWORD = "/" + R_FORGOT_PASSWORD; // "/ind/forgot-password"

// ** Page: Profile **

export const R_PROFILE = R_ROOT + "/" + "profile"; // "ind/profile"
export const ROUTE_PROFILE = "/" + R_PROFILE;      // "/ind/profile"

// ** Page: Stream **
export const R_STREAM = R_ROOT + "/" + "stream"; // "ind/stream"
export const ROUTE_STREAM = "/" + R_STREAM;      // "/ind/stream"

export const E_STREAM_LIST = "list";
export const R_STREAM_LIST = R_STREAM + "/" + E_STREAM_LIST; //  "ind/stream/list"
export const ROUTE_STREAM_LIST = "/" + R_STREAM_LIST;        // "/ind/stream/list"

export const E_STREAM_EDIT = "edit";
export const P_STREAM_ID = "streamId";
export const R_STREAM_EDIT = R_STREAM + "/" + E_STREAM_EDIT; //  "ind/stream/edit" + "/:" + "streamId"
export const ROUTE_STREAM_EDIT = "/" + R_STREAM_EDIT;        // "/ind/stream/edit" + "/:" + "streamId"

export const E_STREAM_CREATE = "create";
export const R_STREAM_CREATE = R_STREAM + "/" + E_STREAM_CREATE; //  "ind/stream/create"
export const ROUTE_STREAM_CREATE = "/" + R_STREAM_CREATE;        // "/ind/stream/create"

// ** Page: Banned users **

export const R_BANNED = R_ROOT + "/" + "banned"; // "ind/banned"
export const ROUTE_BANNED = "/" + R_BANNED;      // "/ind/banned"

// ** Page: Browse **
export const R_BROWSE = R_ROOT + "/" + "browse"; // "ind/browse"
export const ROUTE_BROWSE = "/" + R_BROWSE;      // "/ind/browse"

export const E_BROWSE_LIST = "list";
export const R_BROWSE_LIST = R_BROWSE + "/" + E_BROWSE_LIST; //  "ind/browse/list"
export const ROUTE_BROWSE_LIST = "/" + R_BROWSE_LIST;         // "/ind/browse/list"
export const G_BROWSE_LIVE = "live";
export const G_BROWSE_TAG = "tag";
export const G_BROWSE_PAGE = "page";

export const E_BROWSE_VIEW = "view";
export const P_BROWSE_ID = "streamId";
export const R_BROWSE_VIEW = R_BROWSE + "/" + E_BROWSE_VIEW; //  "ind/browse/view" + "/:" + "streamId"
export const ROUTE_BROWSE_VIEW = "/" + R_BROWSE_VIEW;         // "/ind/browse/view" + "/:" + "streamId"

// ** **


// Route for redirection after login.
export const REDIRECT_AFTER_LOGIN = ROUTE_ABOUT;

export const AUTHENT_REQUIRED = [
    ROUTE_PROFILE,
    ROUTE_STREAM,
    ROUTE_BROWSE_LIST,
    ROUTE_BANNED,
];
export const AUTHENT_DENIED = [
    ROUTE_LOGIN,
    ROUTE_SIGNUP,
];

// ** Main menu
export const MAIN_MENU_LIST = [
    ROUTE_ABOUT,
    ROUTE_LOGIN,
    ROUTE_SIGNUP,
    ROUTE_PROFILE,
    ROUTE_STREAM_LIST,
    ROUTE_STREAM_CREATE,
    ROUTE_BANNED,
    ROUTE_BROWSE_LIST,
];


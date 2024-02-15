// ** Route **
export const R_ROOT = 'ind';

// ** Login **
export const R_LOGIN = R_ROOT + '/' + 'login'; // 'ind/login'
export const ROUTE_LOGIN = '/' + R_LOGIN;      // '/ind/login'

export const R_SIGNUP = R_ROOT + '/' + 'signup'; // 'ind/signup'
export const ROUTE_SIGNUP = '/' + R_SIGNUP;      // '/ind/signup'

export const R_FORGOT_PASSWORD = R_ROOT + '/' + 'forgot-password'; // 'ind/forgot-password'
export const ROUTE_FORGOT_PASSWORD = '/' + R_FORGOT_PASSWORD; // '/ind/forgot-password'

// ** View **
export const R_VIEW = R_ROOT + '/' + 'view'; // 'ind/view'
export const ROUTE_VIEW = '/' + R_VIEW; // '/ind/view'

// ** stream **
export const R_STREAM = R_ROOT + '/' + 'stream'; // 'ind/stream'
export const ROUTE_STREAM = '/' + R_STREAM;

// export const R_STREAM_LIST = 'list';
export const E_STREAM_LIST = 'list';
export const R_STREAM_LIST = R_STREAM + '/' + E_STREAM_LIST; //  'ind/stream/list'
export const ROUTE_STREAM_LIST = '/' + R_STREAM_LIST;        // '/ind/stream/list'

export const E_STREAM_EDIT = 'edit';
export const P_STREAM_ID = 'streamId';
export const R_STREAM_EDIT = R_STREAM + '/' + E_STREAM_EDIT; //  'ind/stream/edit' + '/:' + 'streamId'
export const ROUTE_STREAM_EDIT = '/' + R_STREAM_EDIT;        // '/ind/stream/edit' + '/:' + 'streamId'

// export const R_STREAM_CREATE = 'create';
export const E_STREAM_CREATE = 'create';
export const R_STREAM_CREATE = R_STREAM + '/' + E_STREAM_CREATE; //  'ind/stream/create'
export const ROUTE_STREAM_CREATE = '/' + R_STREAM_CREATE;        // '/ind/stream/create'
// export const ROUTE_STREAM_CREATE = '/' + R_STREAM + '/' + R_STREAM_CREATE;

export const AUTHORIZATION_REQUIRED = [
  ROUTE_STREAM
  // ROUTE_PROFILE,
  // ROUTE_FOLLOWERS,
  // ROUTE_FOLLOWS,
  // ROUTE_BANNED
];
export const AUTHORIZATION_DENIED = [
  ROUTE_LOGIN,
  ROUTE_SIGNUP,
  ROUTE_FORGOT_PASSWORD,
  // ROUTE_CONFIRMATION_REGISTRATION,
  // ROUTE_CONFIRMATION_RECOVERY,
  // ROUTE_CONFIRMATION_FORGOT_PASSWORD,
  // ROUTE_TECHNICAL,
];

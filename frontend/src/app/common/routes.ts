// ** Route **
export const R_ROOT = 'ind';

// ** Login **
export const R_LOGIN = R_ROOT + '/' + 'login'; // 'ind/login'
export const ROUTE_LOGIN = '/' + R_LOGIN; // '/ind/login'

export const R_SIGNUP = R_ROOT + '/' + 'signup'; // 'ind/signup'
export const ROUTE_SIGNUP = '/' + R_SIGNUP; // '/ind/signup'

export const R_FORGOT_PASSWORD = R_ROOT + '/' + 'forgot-password'; // 'ind/forgot-password'
export const ROUTE_FORGOT_PASSWORD = '/' + R_FORGOT_PASSWORD; // '/ind/forgot-password'

// ** View **
export const R_VIEW = R_ROOT + '/' + 'view'; // 'ind/view'
export const ROUTE_VIEW = '/' + R_VIEW; // '/ind/view'

export const AUTHORIZATION_REQUIRED = [
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

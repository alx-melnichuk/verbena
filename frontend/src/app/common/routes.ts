// ** Route **
export const ROUTE_ROOT = '/';

// ** Login **
export const R_LOGIN = 'login';
export const ROUTE_LOGIN = '/' + R_LOGIN;
export const R_SIGNUP = 'signup';
export const ROUTE_SIGNUP = '/' + R_SIGNUP;

export const AUTHORIZATION_REQUIRED = [
  // ROUTE_PROFILE,
  // ROUTE_FOLLOWERS,
  // ROUTE_FOLLOWS,
  // ROUTE_BANNED
];
export const AUTHORIZATION_DENIED = [
  ROUTE_LOGIN,
  ROUTE_SIGNUP,
  // ROUTE_CONFIRMATION_REGISTRATION,
  // ROUTE_CONFIRMATION_RECOVERY,
  // ROUTE_CONFIRMATION_FORGOT_PASSWORD,
  // ROUTE_TECHNICAL,
];

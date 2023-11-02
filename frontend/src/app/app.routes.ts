import { Routes } from '@angular/router';

import { R_FORGOT_PASSWORD, R_LOGIN, R_SIGNUP } from './common/routes';

export const routes: Routes = [
  {
    path: R_LOGIN,
    loadComponent: () => import('./pg-login/pg-login.component').then((c) => c.PgLoginComponent),
  },
  {
    path: R_SIGNUP,
    loadComponent: () => import('./pg-signup/pg-signup.component').then((c) => c.PgSignupComponent),
  },
  {
    path: R_FORGOT_PASSWORD,
    loadComponent: () => import('./pg-forgot-password/pg-forgot-password.component').then((c) => c.PgForgotPasswordComponent),
  },
  {
    path: '',
    loadComponent: () => import('./pg-view/pg-view.component').then((c) => c.PgViewComponent),
  },
  { path: '**', redirectTo: R_LOGIN },
];

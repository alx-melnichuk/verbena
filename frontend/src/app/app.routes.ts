import { Routes } from '@angular/router';

import { R_ABOUT, R_FORGOT_PASSWORD, R_LOGIN, R_PROFILE, R_SIGNUP, R_STREAM, R_VIEW } from './common/routes';
import { authenticationGuard } from './common/authentication.guard';

export const APP_ROUTES: Routes = [
  {
    path: R_ABOUT,
    loadComponent: () => import('./pg-about/pg-about.component').then((c) => c.PgAboutComponent),
  },
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
    path: R_PROFILE,
    loadChildren: () => import('./pg-profile/pg-profile.routes').then((c) => c.PG_PROFILE_ROUTES),
    canActivate: [authenticationGuard],
  },
  {
    path: R_VIEW,
    loadComponent: () => import('./pg-view/pg-view.component').then((c) => c.PgViewComponent),
    canActivate: [authenticationGuard],
  },
  {
    path: R_STREAM,
    loadChildren: () => import('./pg-stream/pg-stream.routes').then(c => c.PG_STREAM_ROUTES),
    canActivate: [authenticationGuard],
  },
  { path: '**', redirectTo: R_ABOUT },
];

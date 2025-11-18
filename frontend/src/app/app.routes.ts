import { Routes } from '@angular/router';

import { R_ABOUT, R_BANNED, R_CONCEPT, R_FORGOT_PASSWORD, R_LOGIN, R_PROFILE, R_SIGNUP, R_STREAM } from './common/routes';
import { authenticationGuard } from './common/authentication.guard';

export const APP_ROUTES: Routes = [
    {
        path: R_ABOUT, // 'ind/about'
        loadChildren: () => import('./pg-about/pg-about.routes').then((c) => c.PG_ABOUT_ROUTES),
    },
    {
        path: R_LOGIN, // 'ind/login'
        loadChildren: () => import('./pg-login/pg-login.routes').then((c) => c.PG_LOGIN_ROUTES),
    },
    {
        path: R_SIGNUP, // 'ind/signup'
        loadChildren: () => import('./pg-signup/pg-signup.routes').then((c) => c.PG_SIGNUP_ROUTES),
    },
    {
        path: R_FORGOT_PASSWORD, // 'ind/forgot-password'
        loadChildren: () => import('./pg-forgot-password/pg-forgot-password.routes').then((c) => c.PG_FORGOT_PASSWORD_ROUTES),
    },
    {
        path: R_PROFILE, // 'ind/profile'
        loadChildren: () => import('./pg-profile/pg-profile.routes').then((c) => c.PG_PROFILE_ROUTES),
        canActivate: [authenticationGuard],
    },
    {
        path: R_STREAM, // 'ind/stream'
        loadChildren: () => import('./pg-stream/pg-stream.routes').then(c => c.PG_STREAM_ROUTES),
        canActivate: [authenticationGuard],
    },
    {
        path: R_CONCEPT, // 'ind/concept'
        loadChildren: () => import('./pg-concept/pg-concept.routers').then(c => c.PG_CONCEPT_ROUTES),
        // Authorization is not required.
    },
    {
        path: R_BANNED, // 'ind/banned'
        loadChildren: () => import('./pg-banned/pg-banned.routers').then(c => c.PG_BANNED_ROUTES),
        canActivate: [authenticationGuard],
    },
    { path: '**', redirectTo: R_ABOUT },
];

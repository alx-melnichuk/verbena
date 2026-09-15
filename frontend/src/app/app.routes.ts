import { Routes } from "@angular/router";
import { authentCanActivateGuard } from "./common/authent-can-activate-guard";
import { authentCanMatchGuard } from "./common/authent-can-match-guard";
import { R_ABOUT, R_BROWSE, R_FORGOT_PASSWORD, R_LOGIN, R_PROFILE, R_SIGNUP, R_STREAM } from "./common/routes";

export const APP_ROUTES: Routes = [
    {
        path: R_ABOUT, // "ind/about"
        loadChildren: () => import("./lib-pg-about/pg-about/pg-about.routes").then((c) => c.PG_ABOUT_ROUTES),
        // Authorization is not required.
    },
    {
        path: R_LOGIN, // "ind/login"
        loadChildren: () => import("./lib-pg-login/pg-login/pg-login.routes").then((c) => c.PG_LOGIN_ROUTES),
        // Authorization is not required.
    },
    {
        path: R_SIGNUP, // "ind/signup"
        loadChildren: () => import("./lib-pg-signup/pg-signup/pg-signup.routes").then((c) => c.PG_SIGNUP_ROUTES),
        // Authorization is not required.
    },
    {
        path: R_FORGOT_PASSWORD, // "ind/forgot-password"
        loadChildren: () => import("./lib-pg-forgot-password/pg-forgot-password/pg-forgot-password.routes")
            .then((c) => c.PG_FORGOT_PASSWORD_ROUTES),
        // Authorization is not required.
    },
    {
        path: R_PROFILE, // "ind/profile"
        loadChildren: () => import("./lib-pg-profile/pg-profile/pg-profile.routes").then((c) => c.PG_PROFILE_ROUTES),
        canActivate: [authentCanActivateGuard], // Authorization is required.
        canMatch: [authentCanMatchGuard],
    },
    {
        path: R_STREAM, // "ind/stream"
        loadChildren: () => import("./lib-pg-stream/pg-stream/pg-stream.routes").then(c => c.PG_STREAM_ROUTES),
        canActivate: [authentCanActivateGuard], // Authorization is required.
        canMatch: [authentCanMatchGuard],
    },
    {
        path: R_BROWSE, // "ind/browse"
        loadChildren: () => import("./lib-pg-browse/pg-browse/pg-browse.routes").then(c => c.PG_BROWSE_ROUTES),
        canActivate: [authentCanActivateGuard], // Authorization is required.
        canMatch: [authentCanMatchGuard],
    },
    { path: "**", redirectTo: R_ABOUT },
];

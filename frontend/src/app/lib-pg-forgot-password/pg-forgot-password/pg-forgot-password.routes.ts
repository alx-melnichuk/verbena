import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment as env } from "../../../environments/environment";
import { LOG_PG_FORGOT_PASSWORD } from "../../common/routes";
import { PgForgotPassword } from "./pg-forgot-password";
import { pgForgotPasswordTranslateResolver } from "./pg-forgot-password-translate.resolver";

// AoT requires an exported function for factories
export function translateLibForgotPasswordHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (env.logLevel & LOG_PG_FORGOT_PASSWORD) { console.info(`translateLibForgotPasswordHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-forgot-password/i18n/", ".json");
};

export const PG_FORGOT_PASSWORD_ROUTES: Routes = [
    {
        path: "",
        component: PgForgotPassword,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibForgotPasswordHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: true,
            }),
        ],
        resolve: { loadTranslate: pgForgotPasswordTranslateResolver },
    },
];

import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment as env } from "../../../environments/environment";
import { LOG_PG_LOGIN } from "../../common/routes";
import { PgLogin } from "./pg-login";
import { pgLoginTranslateResolver } from "./pg-login-translate.resolver";

// AoT requires an exported function for factories
export function translateLibLoginHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (env.logLevel & LOG_PG_LOGIN) { console.info(`translateLibLoginHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-login/i18n/", ".json");
};

export const PG_LOGIN_ROUTES: Routes = [
    {
        path: "",
        component: PgLogin,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibLoginHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: true,
            }),
        ],
        resolve: { loadTranslate: pgLoginTranslateResolver },
    },
];

import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment } from "../../../environments/environment";
import { PgSignup } from "./pg-signup";
import { pgSignupTranslateResolver } from "./pg-signup-translate.resolver";

// AoT requires an exported function for factories
export function translateLibSignupHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (environment.logLevel > 0) { console.log(`translateLibSignupHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-signup/i18n/", ".json");
};

export const PG_SIGNUP_ROUTES: Routes = [
    {
        path: "",
        component: PgSignup,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibSignupHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: false,
            }),
        ],
        resolve: { loadTranslate: pgSignupTranslateResolver },
    },
];

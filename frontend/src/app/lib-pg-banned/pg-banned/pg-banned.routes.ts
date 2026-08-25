import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment } from "../../../environments/environment";
import { PgBanned } from "./pg-banned";
import { pgBannedTranslateResolver } from "./pg-banned-translate.resolver";
import { pgBannedUsersResolver } from "./pg-banned-users.resolver";

// AoT requires an exported function for factories
export function translateLibBannedHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (environment.logLevel > 0) { console.info(`translateLibBannedHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-banned/i18n/", ".json");
};

export const PG_BANNED_ROUTES: Routes = [
    {
        path: "",
        component: PgBanned,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibBannedHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: true,
            }),
        ],
        resolve: {
            loadTranslate: pgBannedTranslateResolver,
            blockedUsers: pgBannedUsersResolver,
        },
    },
];

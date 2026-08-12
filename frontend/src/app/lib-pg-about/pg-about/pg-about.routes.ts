import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment } from "../../../environments/environment";
import { PgAbout } from "./pg-about";
import { pgAboutResolver } from "./pg-about.resolver";

// AoT requires an exported function for factories
export function translateLibAboutHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (environment.logLevel > 0) { console.info(`translateLibAboutHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-about/i18n/", ".json");
};

export const PG_ABOUT_ROUTES: Routes = [
    {
        path: "",
        component: PgAbout,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibAboutHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: true,
            }),
        ],
        resolve: { aboutTranslate: pgAboutResolver },
    },
];

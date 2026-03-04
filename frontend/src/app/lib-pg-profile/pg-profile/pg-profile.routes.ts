import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment } from "../../../environments/environment";
import { PgProfile } from "./pg-profile";
import { pgProfileConfigResolver } from "./pg-profile-config.resolver";
import { pgProfileDtoResolver } from "./pg-profile-dto.resolver";
import { pgProfileTranslateResolver } from "./pg-profile-translate.resolver";

// AoT requires an exported function for factories
export function translateLibProfileHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (environment.logLevel > 0) { console.log(`translateLibProfileHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-profile/i18n/", ".json");
};

export const PG_PROFILE_ROUTES: Routes = [
    {
        path: "",
        component: PgProfile,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibProfileHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: false,
            }),
        ],
        resolve: {
            loadTranslate: pgProfileTranslateResolver,
            profileDto: pgProfileDtoResolver,
            profileConfigDto: pgProfileConfigResolver,
        },
    },
];

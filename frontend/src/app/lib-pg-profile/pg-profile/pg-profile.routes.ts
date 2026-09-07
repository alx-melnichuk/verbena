import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment as env } from "../../../environments/environment";
import { E_PROFILE_SETTINGS, E_PROFILE_DETAILS, LOG_PG_PROFILE } from "../../common/routes";
import { unsavedDeActivatePageGuard } from "../../common/unsaved-de-activate-page-guard";
import { PgProfile } from "./pg-profile";
import { pgProfileConfigResolver } from "./pg-profile-config.resolver";
import { PgProfileDetails } from "../pg-profile-details/pg-profile-details";
import { pgProfileDtoResolver } from "./pg-profile-dto.resolver";
import { PgProfileSettings } from "../pg-profile-settings/pg-profile-settings";
import { pgProfileTranslateResolver } from "./pg-profile-translate.resolver";

// AoT requires an exported function for factories
export function translateLibProfileHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (env.logLevel & LOG_PG_PROFILE) { console.info(`translateLibProfileHttpLoaderFactory()`); }
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
        },
        children: [
            {
                path: E_PROFILE_DETAILS, // "ind/profile/details"
                component: PgProfileDetails,
                resolve: {
                    profileDto: pgProfileDtoResolver,
                    profileConfigDto: pgProfileConfigResolver,
                },
                canDeactivate: [unsavedDeActivatePageGuard],
            },
            {
                path: E_PROFILE_SETTINGS, // "ind/profile/settings"
                component: PgProfileSettings,
                resolve: {
                    profileDto: pgProfileDtoResolver,
                },
                canDeactivate: [unsavedDeActivatePageGuard],
            },
        ]
    },
];

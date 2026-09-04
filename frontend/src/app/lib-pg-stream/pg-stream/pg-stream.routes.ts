import { Routes } from "@angular/router";
import { provideHttpClient, withInterceptorsFromDi, HttpClient } from "@angular/common/http";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment as env } from "../../../environments/environment";
import { E_STREAM_LIST, E_STREAM_EDIT, P_STREAM_ID, E_STREAM_CREATE, LOG_PG_STREAM } from "../../common/routes";
import { unsavedDeActivatePageGuard } from "../../common/unsaved-de-activate-page-guard";
import { PgStreamEdit } from "../pg-stream-edit/pg-stream-edit";
import { PgStreamList } from "../pg-stream-list/pg-stream-list";
import { PgStream } from "./pg-stream";
import { pgStreamConfigResolver } from "./pg-stream-config.resolver";
import { pgStreamDtoResolver } from "./pg-stream-dto.resolver";
import { pgStreamTranslateResolver } from "./pg-stream-translate.resolver";

// AoT requires an exported function for factories
export function translateLibStreamHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (env.logLevel & LOG_PG_STREAM) { console.info(`translateLibStreamHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-stream/i18n/", ".json");
};

export const PG_STREAM_ROUTES: Routes = [
    {
        path: "",
        component: PgStream,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibStreamHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: false,
            }),
        ],
        resolve: { loadTranslate: pgStreamTranslateResolver },
        children: [
            {
                path: E_STREAM_LIST, // "ind/stream/list"
                component: PgStreamList,
            },
            {
                path: E_STREAM_EDIT + "/:" + P_STREAM_ID, // "ind/stream/edit/:streamId"
                pathMatch: "prefix",
                component: PgStreamEdit,
                resolve: {
                    streamDto: pgStreamDtoResolver,
                    streamConfigDto: pgStreamConfigResolver,
                },
                canDeactivate: [unsavedDeActivatePageGuard],
            },
            {
                path: E_STREAM_CREATE, // "ind/stream/create"
                pathMatch: "full",
                component: PgStreamEdit,
                resolve: {
                    streamDto: pgStreamDtoResolver,
                    streamConfigDto: pgStreamConfigResolver,
                },
                canDeactivate: [unsavedDeActivatePageGuard],
            },
        ]
    },
];

import { HttpClient, provideHttpClient, withInterceptorsFromDi } from "@angular/common/http";
import { Routes } from "@angular/router";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment as env } from "../../../environments/environment";
import { E_BROWSE_LIST, E_BROWSE_DETAILS, LOG_PG_BROWSE, P_BROWSE_ID } from "../../common/routes";
import { PgBrowseDetails } from "../pg-browse-details/pg-browse-details";
import { PgBrowseList } from "../pg-browse-list/pg-browse-list";
import { PgBrowse } from "./pg-browse";
import { pgBrowseTranslateResolver } from "./pg-browse-translate.resolver";
import { pgChatMessagesResolver } from "./pg-chat-messages.resolver";
import { pgUserResolver } from "./pg-user.resolver";
import { pgBrowseStreamResolver } from "./pg-browse-stream.resolver";
import { pgAccessTokenResolver } from "./pg-access-token.resolver";


// AoT requires an exported function for factories
export function translateLibBrowseHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (env.logLevel & LOG_PG_BROWSE) { console.info(`translateLibBrowseHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./lib-pg-browse/i18n/", ".json");
};

export const PG_BROWSE_ROUTES: Routes = [
    {
        path: "",
        component: PgBrowse,
        providers: [
            provideHttpClient(withInterceptorsFromDi()),
            provideTranslateService({
                loader: {
                    provide: TranslateLoader,
                    useFactory: translateLibBrowseHttpLoaderFactory,
                    deps: [HttpClient],
                },
                extend: true,
                isolate: false,
            }),
        ],
        resolve: {
            loadTranslate: pgBrowseTranslateResolver
        },
        children: [
            {
                path: E_BROWSE_LIST, // "ind/browse/list"
                component: PgBrowseList,
            },
            {
                path: E_BROWSE_DETAILS + "/:" + P_BROWSE_ID, // "ind/browse/details/:streamId"
                pathMatch: "prefix",
                component: PgBrowseDetails,
                resolve: {
                    user: pgUserResolver,
                    accessToken: pgAccessTokenResolver,
                    chatMsgList: pgChatMessagesResolver,
                    browseStream: pgBrowseStreamResolver,
                },
            },
        ]
    },
];

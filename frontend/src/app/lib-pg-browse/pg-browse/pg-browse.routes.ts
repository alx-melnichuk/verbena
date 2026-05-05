import { HttpClient, provideHttpClient, withInterceptorsFromDi } from "@angular/common/http";
import { Routes } from "@angular/router";
import { provideTranslateService, TranslateLoader } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment } from "../../../environments/environment";
import { E_BROWSE_LIST, E_BROWSE_VIEW, P_BROWSE_ID } from "../../common/routes";
import { PgBrowseList } from "../pg-browse-list/pg-browse-list";
import { PgBrowseView } from "../pg-browse-view/pg-browse-view";
import { PgBrowse } from "./pg-browse";
import { pgBrowseTranslateResolver } from "./pg-browse-translate.resolver";
import { pgChatMessagesResolver } from "./pg-chat-messages.resolver";
import { pgUserResolver } from "./pg-user.resolver";
import { pgBrowseStreamResolver } from "./pg-browse-stream.resolver";
import { pgAccessTokenResolver } from "./pg-access-token.resolver";


// AoT requires an exported function for factories
export function translateLibBrowseHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (environment.logLevel > 0) { console.info(`translateLibBrowseHttpLoaderFactory()`); }
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
        resolve: { loadTranslate: pgBrowseTranslateResolver },
        children: [
            {
                path: E_BROWSE_LIST, // "ind/browse/list"
                component: PgBrowseList,
            },
            {
                path: E_BROWSE_VIEW + "/:" + P_BROWSE_ID, // "ind/browse/view/:streamId"
                pathMatch: "prefix",
                component: PgBrowseView,
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

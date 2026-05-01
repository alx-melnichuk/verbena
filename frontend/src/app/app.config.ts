import { HTTP_INTERCEPTORS, HttpClient, HttpErrorResponse, provideHttpClient, withInterceptorsFromDi } from "@angular/common/http";
import {
    ApplicationConfig, importProvidersFrom, inject, provideAppInitializer,
    provideBrowserGlobalErrorListeners, provideZonelessChangeDetection
} from "@angular/core";
import { MAT_DATE_LOCALE, DateAdapter, MAT_DATE_FORMATS } from "@angular/material/core";
import { MatDialogModule, MatDialog } from "@angular/material/dialog";
import { MAT_FORM_FIELD_DEFAULT_OPTIONS } from "@angular/material/form-field";
import { MatSnackBarModule } from "@angular/material/snack-bar";
import { provideRouter, Router } from "@angular/router";
import { provideTranslateService, TranslateLoader, TranslateService } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment } from "../environments/environment";
import { AppDateAdapter, APP_DATE_FORMATS } from "./app-date-adapter";
import { APP_ROUTES } from "./app.routes";
import { AuthorizationInterceptor } from "./common/authorization.interceptor";
import { APP_DATE_TIME_FORMAT_CONFIG } from "./common/date-time-format-pipe";
import { LOCALE_DEFAULT, LocaleSrv } from "./common/locale-srv";
import { RedirectSrv } from "./common/redirect-srv";
import { AUTHENT_REQUIRED, ROUTE_LOGIN } from "./common/routes";
import { SessionSrv } from "./common/session-srv";
import { DialogSrv } from "./lib-dialog/dialog-srv";
import { UserSrv } from "./lib-user/user-srv";
import { DateUtil } from "./utils/date.utils";
import { NavigatorUtil } from "./utils/navigator.util";
import { UserDto } from "./lib-user/user-dto";

// AoT requires an exported function for factories
export function translateAppHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (environment.logLevel > 0) { console.log(`translateAppHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./i18n/", ".json");
};

export const appConfig: ApplicationConfig = {
    providers: [
        provideBrowserGlobalErrorListeners(),

        provideZonelessChangeDetection(),
        // provideZoneChangeDetection({ eventCoalescing: true }),
        provideRouter(APP_ROUTES),

        provideHttpClient(withInterceptorsFromDi()),
        provideTranslateService({
            loader: {
                provide: TranslateLoader,
                useFactory: translateAppHttpLoaderFactory,
                deps: [HttpClient],
            },
        }),
        {
            provide: HTTP_INTERCEPTORS,
            useClass: AuthorizationInterceptor,
            multi: true,
        },
        {
            provide: MAT_DATE_LOCALE,
            useValue: LOCALE_DEFAULT
        },
        {
            provide: DateAdapter,
            useClass: AppDateAdapter,
            deps: [MAT_DATE_LOCALE]
        },
        {
            provide: MAT_DATE_FORMATS,
            useValue: APP_DATE_FORMATS
        },
        {
            provide: APP_DATE_TIME_FORMAT_CONFIG,
            useValue: { afterFormat: DateUtil.afterFormat }
        },

        LocaleSrv, // "1-Srv"
        provideAppInitializer(() => {
            const localeSrv = inject(LocaleSrv);
            const translate = inject(TranslateService);

            // Download translations before starting the application.
            translate.addLangs(localeSrv.localeList);
            translate.setDefaultLang(localeSrv.localeDefault);

            const localeFromLocalStorage = localeSrv.getFromLocalStorage();
            const value = localeSrv.findLocale(localeSrv.localeList, localeFromLocalStorage || NavigatorUtil.getBrowserLocale() || null)
                || null;
            if (environment.logLevel > 0) { console.log(`provideAppInitializer(localeSrv) locale.setLocale(${value});`); }
            return localeSrv.setLocale(value);
        }),
        RedirectSrv,
        SessionSrv,
        UserSrv,
        provideAppInitializer(() => {
            const redirectSrv: RedirectSrv = inject(RedirectSrv);
            const router = inject(Router);
            const sessionSrv = inject(SessionSrv);
            const userSrv = inject(UserSrv);

            const currentRoute = window.location.pathname;
            const isAuthentRequired = AUTHENT_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
            const isAuthentDenied = AUTHENT_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
            const isNotAuthentDeniedAndHasAccessToken = !isAuthentDenied && !!sessionSrv.getAccessToken();
            if (environment.logLevel > 0) {
                const s1 = `isNotAuthentDeniedAndHasAccessToken: ${isNotAuthentDeniedAndHasAccessToken}`;
                console.log(`provideAppInitializer(userSrv) isAuthentRequired: ${isAuthentRequired}, ${s1}`);
            }

            if (isAuthentRequired || isNotAuthentDeniedAndHasAccessToken) {
                if (environment.logLevel > 0) { console.log(`provideAppInitializer(userSrv) userSrv.getCurrentUser()...`); }
                return userSrv.getCurrentUser()
                    .then((response: UserDto | HttpErrorResponse | undefined) => {
                        const user = response as UserDto;
                        if (environment.logLevel > 0) {
                            console.log(`provideAppInitializer(userSrv) userSrv.getCurrentUser()...Ok`
                                + ` user.id: ${user.id}, user.nickname: ${user.nickname} `);
                        }
                        sessionSrv.setUser(user);
                        return Promise.resolve();
                    })
                    .catch((err: HttpErrorResponse) => {
                        if (isNotAuthentDeniedAndHasAccessToken) {
                            // Save the link address to navigate to after login.
                            redirectSrv.setUrlAfterLogin(window.location.pathname);
                            window.setTimeout(() => router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
                        }
                    });
            } else {
                return Promise.resolve();
            }

        }),
        // Represents the default options for form fields.
        {
            provide: MAT_FORM_FIELD_DEFAULT_OPTIONS,
            useValue: {
                // Default form field appearance style.
                appearance: "outline",
                color: "primary",
                // Whether the required marker should be hidden by default.
                // hideRequiredMarker?: boolean,
                // Whether the label for form fields should by default float "always", "never", or "auto" (only when necessary).
                // floatLabel?: FloatLabelType, 
                // Whether the form field should reserve space for one line by default.
                subscriptSizing: "fixed", // "fixed" | "dynamic"
            }
        },
        importProvidersFrom(MatDialogModule, MatSnackBarModule),
        {
            provide: DialogSrv,
            useClass: DialogSrv,
            deps: [MatDialog],
        },
    ]
};

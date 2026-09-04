import { HTTP_INTERCEPTORS, HttpClient, HttpErrorResponse, provideHttpClient, withInterceptorsFromDi } from "@angular/common/http";
import {
    ApplicationConfig, importProvidersFrom, inject, InjectionToken, provideAppInitializer,
    provideBrowserGlobalErrorListeners, provideZonelessChangeDetection
} from "@angular/core";
import { MAT_DATE_LOCALE, DateAdapter, MAT_DATE_FORMATS } from "@angular/material/core";
import { MatDialogModule, MatDialog } from "@angular/material/dialog";
import { MAT_FORM_FIELD_DEFAULT_OPTIONS, MatFormFieldDefaultOptions } from "@angular/material/form-field";
import { MatSnackBarModule } from "@angular/material/snack-bar";
import { provideRouter, Router } from "@angular/router";
import { provideTranslateService, TranslateLoader, TranslateService } from "@ngx-translate/core";
import { TranslateHttpLoader } from "@ngx-translate/http-loader";
import { environment as env } from "../environments/environment";
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
import { AppearanceSvr } from "./common/appearance-svr";
import { ColorSchemeSrv } from "./common/color-scheme-srv";

// AoT requires an exported function for factories
export function translateAppHttpLoaderFactory(httpClient: HttpClient): TranslateHttpLoader {
    if (env.logLevel & 4) { console.info(`translateAppHttpLoaderFactory()`); }
    return new TranslateHttpLoader(httpClient, "./i18n/", ".json");
};

// Define configuration MatFormFieldDefaultOptions.
export const appFormFieldDefaultOptions: MatFormFieldDefaultOptions = {
    // Default form field appearance style.
    appearance: "outline", // "fill"
    color: "primary",
    // Whether the required marker should be hidden by default.
    // hideRequiredMarker?: boolean,
    // Whether the label for form fields should by default float "always", "never", or "auto" (only when necessary).
    // floatLabel?: FloatLabelType, 
    // Whether the form field should reserve space for one line by default.
    subscriptSizing: "fixed", // "fixed" | "dynamic"
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
            useClass: AuthorizationInterceptor, // "4-Srv"
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
            if (env.logLevel & 4) { console.info(`provideAppInitializer(localeSrv) locale.setLocale(${value});`); }
            return localeSrv.setLocale(value);
        }),
        SessionSrv, // "2-Srv"
        UserSrv,  // "3-Srv"
        RedirectSrv, // "5-Srv"
        AppearanceSvr, // "6-Srv"
        provideAppInitializer(() => {
            const redirectSrv: RedirectSrv = inject(RedirectSrv);
            const router = inject(Router);
            const sessionSrv = inject(SessionSrv);
            const userSrv = inject(UserSrv);
            const appearanceSvr = inject(AppearanceSvr);

            const currentRoute = window.location.pathname;
            const isAuthentRequired = AUTHENT_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
            const isAuthentDenied = AUTHENT_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
            const isNotAuthentDeniedAndHasAccessToken = !isAuthentDenied && !!sessionSrv.getAccessToken();
            if (env.logLevel & 4) {
                const s1 = `isNotAuthentDeniedAndHasAccessToken: ${isNotAuthentDeniedAndHasAccessToken}`;
                console.info(`provideAppInitializer(userSrv) isAuthentRequired: ${isAuthentRequired}, ${s1}`);
            }

            if (isAuthentRequired || isNotAuthentDeniedAndHasAccessToken) {
                if (env.logLevel & 4) { console.info(`provideAppInitializer(userSrv) userSrv.getCurrentUser()...`); }
                return userSrv.getCurrentUser()
                    .then((response: UserDto | HttpErrorResponse | undefined) => {
                        const user = response as UserDto;
                        if (env.logLevel & 4) {
                            console.info(`provideAppInitializer(userSrv) userSrv.getCurrentUser()...Ok`
                                + ` user.id: ${user.id}, user.nickname: ${user.nickname} `);
                        }

                        const settings = { ...user.settings };
                        user.settings = undefined;
                        sessionSrv.setUser(user);
                        sessionSrv.setSettings(settings);
                        appearanceSvr.setSettings(settings);

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
            useValue: appFormFieldDefaultOptions
        },
        importProvidersFrom(MatDialogModule, MatSnackBarModule),
        {
            provide: DialogSrv,
            useClass: DialogSrv,
            deps: [MatDialog],
        },
        ColorSchemeSrv, // "7-Srv"
    ]
};

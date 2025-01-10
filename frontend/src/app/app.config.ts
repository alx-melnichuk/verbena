import { HTTP_INTERCEPTORS, HttpClient, provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { APP_INITIALIZER, ApplicationConfig, ErrorHandler, importProvidersFrom, provideZoneChangeDetection } from '@angular/core';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { MatDialogModule } from '@angular/material/dialog';
import { MatSnackBarModule } from '@angular/material/snack-bar';
import { DateAdapter, MAT_DATE_FORMATS, MAT_DATE_LOCALE } from '@angular/material/core';
import { TranslateHttpLoader } from '@ngx-translate/http-loader';
import { TranslateLoader, TranslateModule } from '@ngx-translate/core';

import { APP_DATE_FORMATS, AppDateAdapter } from './app-date-adapter';
import { AppErrorHandler } from './app-error-handler';
import { APP_ROUTES      } from './app.routes';
import { AuthorizationInterceptor    } from './common/authorization.interceptor';
import { ENV_IS_PROD                 } from './common/constants';
import { APP_DATE_TIME_FORMAT_CONFIG } from './common/date-time-format.pipe';
import { InitializationService       } from './common/initialization.service';
import { LocaleService               } from './common/locale.service';
import { DateUtil        } from './utils/date.utils';

// AoT requires an exported function for factories
export const TRANSLATE_LOADER_FACTORY = (httpClient: HttpClient): TranslateHttpLoader => {
  if (!ENV_IS_PROD) { console.log(`HTTP_LOADER_FACTORY()`); }
  return new TranslateHttpLoader(httpClient, './assets/i18n/', '.json');
};

export const INITIALIZE_TRANSLATE_FACTORY = (initializationService: InitializationService): any => {
  return (): Promise<any> => initializationService.initTranslate();
};

export const INITIALIZE_AUTHENTICATION_USER_FACTORY = (initializationService: InitializationService): any => {
  return (): Promise<any> => initializationService.initSession();
};


export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(APP_ROUTES),
    provideAnimationsAsync(),
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideHttpClient(withInterceptorsFromDi()),
    importProvidersFrom([
      TranslateModule.forRoot({
        loader: {
          provide: TranslateLoader,
          useFactory: TRANSLATE_LOADER_FACTORY,
          deps: [HttpClient],
        },
      })
    ]),
    LocaleService,
    InitializationService,
    {
      provide: HTTP_INTERCEPTORS,
      useClass: AuthorizationInterceptor,
      multi: true,
    },
    {
      provide: MAT_DATE_LOCALE,
      useValue: 'en'
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
    importProvidersFrom(MatDialogModule, MatSnackBarModule),
    {
      provide: APP_INITIALIZER,
      deps: [InitializationService],
      useFactory: INITIALIZE_AUTHENTICATION_USER_FACTORY,
      multi: true,
    },
    {
      provide: APP_INITIALIZER,
      deps: [InitializationService],
      useFactory: INITIALIZE_TRANSLATE_FACTORY,
      multi: true,
    },
    { // Handling the error "Loading chunk [\d]+ failed"
      provide: ErrorHandler,
      useClass: AppErrorHandler
    },
  ]
};

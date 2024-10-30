import { HTTP_INTERCEPTORS, HttpClient, provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { APP_INITIALIZER, ApplicationConfig, ErrorHandler, importProvidersFrom } from '@angular/core';
import { provideAnimations } from '@angular/platform-browser/animations';
import { provideRouter } from '@angular/router';
import { MatDialogModule } from '@angular/material/dialog';
import { MatSnackBarModule } from '@angular/material/snack-bar';
import { DateAdapter, MAT_DATE_FORMATS, MatNativeDateModule } from '@angular/material/core';
import { TranslateHttpLoader } from '@ngx-translate/http-loader';
import { TranslateLoader, TranslateModule } from '@ngx-translate/core';

import { APP_DATE_FORMATS, AppDateAdapter } from './app-date-adapter';
import { APP_ROUTES } from './app.routes';
import { AuthorizationInterceptor } from './common/authorization.interceptor';
import { InitializationService } from './common/initialization.service';
import { AppErrorHandler } from './app-error-handler';

// AoT requires an exported function for factories
export const HTTP_LOADER_FACTORY = (httpClient: HttpClient): TranslateHttpLoader => {
  console.log(`HTTP_LOADER_FACTORY()`); // #
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
    provideAnimations(),
    InitializationService,
    {
      provide: HTTP_INTERCEPTORS,
      useClass: AuthorizationInterceptor,
      multi: true,
    },
    provideHttpClient(withInterceptorsFromDi()),
    importProvidersFrom(
      TranslateModule.forRoot({
        loader: {
          provide: TranslateLoader,
          deps: [HttpClient],
          useFactory: HTTP_LOADER_FACTORY,
        },
      })
    ),
    importProvidersFrom(MatDialogModule, MatSnackBarModule, MatNativeDateModule),
    {
      provide: DateAdapter,
      useClass: AppDateAdapter
    },
    {
      provide: MAT_DATE_FORMATS,
      useValue: APP_DATE_FORMATS
    },
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
  ],
};

import { HTTP_INTERCEPTORS, HttpClient, provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { APP_INITIALIZER, ApplicationConfig, importProvidersFrom } from '@angular/core';
import { provideAnimations } from '@angular/platform-browser/animations';
import { provideRouter } from '@angular/router';
import { MatDialogModule } from '@angular/material/dialog';
import { MatSnackBarModule } from '@angular/material/snack-bar';
import { DateAdapter, MAT_DATE_FORMATS, MatNativeDateModule, NativeDateAdapter } from '@angular/material/core';
import { formatDate } from '@angular/common';
import { TranslateHttpLoader } from '@ngx-translate/http-loader';
import { TranslateLoader, TranslateModule } from '@ngx-translate/core';

import { APP_ROUTES } from './app.routes';
import { AuthorizationInterceptor } from './common/authorization.interceptor';
import { InitializationService } from './common/initialization.service';

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

class AppDateAdapter extends NativeDateAdapter {
  override format(date: Date, displayFormat: Object): string {
    if (displayFormat === 'app-input') {
      return formatDate(date,'dd-MM-yyyy', this.locale);;
    } else {
      return super.format(date, displayFormat);
    }
  }
  override parse(value: any, parseFormat: any): Date | null {
    let result: Date | null = null;
    const value_type = typeof value;
    if (value_type == 'number') {
      result = new Date(value);
    } else if (value_type == 'string') {
      const data = value.slice(6, 10) + '-' + value.slice(3, 5) + '-' + value.slice(0, 2);
      result = new Date(Date.parse(data));
    }
    return result;
  }
}

export const APP_DATE_FORMATS = {
  parse: {
    dateInput: 'DD-MM-YYYY',
  },
  display: {
    // Property in display section is the date format in which displays the date in input box.
    dateInput: 'app-input',
    // Property in display section is the date format in which calendar displays the month-year label.
    monthYearLabel: {year: 'numeric', month: 'short'},
    // Related to Accessibility (a11y)
    dateA11yLabel: {year: 'numeric', month: 'long', day: 'numeric'},
    monthYearA11yLabel: {year: 'numeric', month: 'long'},
  }
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
      useFactory: INITIALIZE_TRANSLATE_FACTORY,
      multi: true,
    },
    {
      provide: APP_INITIALIZER,
      deps: [InitializationService],
      useFactory: INITIALIZE_AUTHENTICATION_USER_FACTORY,
      multi: true,
    },
  ],
};

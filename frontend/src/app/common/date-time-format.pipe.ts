import { inject, Pipe, PipeTransform } from '@angular/core';

import {InjectionToken} from '@angular/core';

/*
  The Intl.DateTimeFormat object enables language-sensitive date and time formatting.

  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/DateTimeFormat
  Intl.DateTimeFormatOptions

  dateStyle?: 'full' | 'long' | 'medium' | 'short';
    The date formatting style to use. Possible values are:
    //                   (en-us)             |  (de-de)
    - 'full'   E.g., Monday, January 6, 2020 | Montag, 6. Januar 2025
    - 'long'   E.g., January 6, 2025         | 6. Januar 2025 
    - 'medium' E.g., Jan 6, 2025             | 06.01.2025, 
    - 'short'  E.g., 1/6/25                  | 06.01.25
    It expands to styles for weekday, day, month, year, and era, with the exact combination of values depending on the locale.

  year?: 'numeric' | '2-digit';
    The representation of the year. Possible values are "numeric" and "2-digit".

  month?: 'numeric' | '2-digit' | 'long' | 'short';
    The representation of the month. Possible values are:
    - 'numeric' E.g., 3
    - '2-digit' E.g., 03
    - 'long'    E.g., March
    - 'short'   E.g., Mar
    - 'narrow'  E.g., M). 
    Two months may have the same narrow style for some locales (e.g. May's narrow style is also M).

  day?: 'numeric' | '2-digit';
    The representation of the day. Possible values are "numeric" and "2-digit".

  timeStyle?: 'full' | 'long' | 'medium' | 'short';
    The time formatting style to use. Possible values are:
    //                (en-us)   |  (de-de)
    - 'full'   E.g., 1:20:00 PM | 13:20:00
    - 'long'   E.g., 1:20:00 PM | 13:20:00
    - 'medium' E.g., 1:20:00 PM | 13:20:00
    - 'short'  E.g., 1:20 PM    | 13:20
    It expands to styles for hour, minute, second, and timeZoneName, with the exact combination of values depending on the locale.

  hour12?: boolean;
    Use 12 or 24 hour time format.

  hour?: 'numeric' | '2-digit';  
    The representation of the hour. Possible values are "numeric" and "2-digit".
  
  minute?: 'numeric' | '2-digit';
    The representation of the minute. Possible values are "numeric" and "2-digit".

  second?: 'numeric' | '2-digit';
    The representation of the second. Possible values are "numeric" and "2-digit".    

  fractionalSecondDigits?: 1 | 2 | 3;
    The number of digits used to represent fractions of a second (any additional digits are truncated).
    Possible values are from 1 to 3.

  timeZone?: string;
    The time zone to use. Time zone names correspond to the Zone and Link names of the IANA Time Zone Database, 
    such as "UTC", "Asia/Shanghai", "Asia/Kolkata", and "America/New_York". Additionally, time zones can be given 
    as UTC offsets in the format "±hh:mm", "±hhmm", or "±hh", for example as "+01:00", "-2359", or "+23".
    The default is the runtime's default time zone.

  timeZoneName?: 'long' | 'short' | 'shortOffset' | 'longOffset' | 'shortGeneric' | 'longGeneric';
    The localized representation of the time zone name. Possible values are:
    - 'long'         Long localized form (e.g., Pacific Standard Time, Nordamerikanische Westküsten-Normalzeit)
    - 'short'        Short localized form (e.g.: PST, GMT-8)
    - 'shortOffset'  Short localized GMT format (e.g., GMT-8)
    - 'longOffset'   Long localized GMT format (e.g., GMT-08:00)
    - 'shortGeneric' Short generic non-location format (e.g.: PT, Los Angeles Zeit).
    - 'longGeneric'  Long generic non-location format (e.g.: Pacific Time, Nordamerikanische Westküstenzeit)

    The default value for each date-time component option is undefined, but if all component properties are 
    undefined, then year, month, and day default to "numeric". If any of the date-time component options is 
    specified, then dateStyle and timeStyle must be undefined.

  Note: dateStyle and timeStyle can be used with each other, but not with other date-time component options 
  (e.g. weekday, hour, month, etc.).
*/

export declare type DateTimeAfterFormatFn = (
  value: string | null | undefined,
  locale?: string | null | undefined,
  options?: Intl.DateTimeFormatOptions | null | undefined
) => string | null;

export type DateTimeFormatConfig = {
  locale?: string | null | undefined,
  options?: Intl.DateTimeFormatOptions | null | undefined,
  afterFormat?: DateTimeAfterFormatFn | null | undefined,
};

export const APP_DATE_TIME_FORMAT_CONFIG = new InjectionToken<DateTimeFormatConfig>('app-date-time-format-config');

/**
 * value: string | Date | number | null | undefined,
 *  - string Date string in ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
 *  - Date Date object
 *  - number Date as a number
 *  For "string" or "number", the date value is defined as new Date(value).
 */
@Pipe({
  name: 'dateTimeFormat',
  standalone: true
})
export class DateTimeFormatPipe implements PipeTransform {

  private readonly dtfc: DateTimeFormatConfig | null = inject(APP_DATE_TIME_FORMAT_CONFIG, { optional: true });

  transform(
    value: string | Date | number | null | undefined,
    locale?: string | null | undefined,
    options?: Intl.DateTimeFormatOptions | null | undefined,
  ): string | null {
    let result: string | null = null;
    const valueDate: Date | null = typeof value == 'number'
      ? (!isNaN(value) ? new Date(value) : null)
      : (typeof value == 'string' ? new Date(value) 
        : (value as Date));
    
    if (valueDate != null) {
      const opts = options != null && !!Object.keys(options).length ? options 
        : this.dtfc?.options != null && !!Object.keys(this.dtfc?.options).length ? this.dtfc?.options
          : undefined;
        
      const dtf = new Intl.DateTimeFormat(locale || this.dtfc?.locale || undefined, opts);
      result = dtf.format(valueDate);
      if (!!this.dtfc?.afterFormat) {
        result = this.dtfc.afterFormat(result, locale || window.navigator.language, options);
      }
    }
    return result;
  }
}
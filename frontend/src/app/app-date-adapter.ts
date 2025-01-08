import { Injectable } from "@angular/core";
import { NativeDateAdapter } from "@angular/material/core";

import { DateUtil } from "./utils/date.utils";

export const APP_DATE_FORMATS = {
  parse: {
    dateInput: null,
  },
  display: {
    // Property in display section is the date format in which displays the date in input box.
    dateInput: {year: 'numeric', month: 'numeric', day: 'numeric'},
    // Property in display section is the date format in which calendar displays the month-year label.
    monthYearLabel: {year: 'numeric', month: 'long'},
    // Related to Accessibility (a11y)
    dateA11yLabel: {year: 'numeric', month: 'long', day: 'numeric'},
    monthYearA11yLabel: {year: 'numeric', month: 'long'},
  }
};

@Injectable()
export class AppDateAdapter extends NativeDateAdapter {
  
  protected formatParts: Intl.DateTimeFormatPart[] = [];

  override parse(value: any, parseFormat: any): Date | null {
    let result: Date | null = null;
    const value_type = typeof value;
    if (value_type == 'number') {
      result = new Date(value);
    } else if (value_type == 'string') {
      result = this.parseFromStringWithLocale(value, this.formatParts);
    }
    return result;
  }

  override format(date: Date, displayFormat: Object): string {
    let result = super.format(date, displayFormat);
    if (!!result) {
      result = DateUtil.afterFormat(result, this.locale || window.navigator.language, displayFormat) as string;
    }
    return result;
  }

  override setLocale(locale: any) {
    super.setLocale(locale);
    // Define an array of formatting literals.
    const formatter = new Intl.DateTimeFormat(locale || undefined);
    this.formatParts = formatter.formatToParts(new Date(Date.UTC(2000, 2, 1, 0, 0, 0, 0)));
  }

  // ** Private API **

  private parseFromStringWithLocale(valueStr: string, formatParts: Intl.DateTimeFormatPart[]): Date | null {
    let result: Date | null = null;
    if (valueStr?.length > 0 && formatParts?.length > 0) {
      const res: {[key: string]: any} = { year: -1, month: -1, day: -1 };
      let key = ''; let val = valueStr;  let index = -1;
      for (let idx = 0; idx < formatParts.length; idx++) {
        const { type, value } = formatParts[idx];
        if (type == 'literal') {
          index = val.indexOf(value);
          if (index == -1) {
            break;
          }
          res[key] = parseInt(val.slice(0, index), 10);
          val = val.slice(index + 1);
        }
        key = type;
      }
      if (!!key && key != 'literal' && index != -1 && !!val) {
        res[key] = parseInt(val, 10);
      }
      if (res['year'] != -1 && res['month'] != -1 && res['day'] != -1) {
        result = this.createDate(res['year'], res['month'] - 1, res['day']);
      }
    }
    return result;
  }
}

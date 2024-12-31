import { formatDate } from "@angular/common";
import { Injectable } from "@angular/core";
import { NativeDateAdapter } from "@angular/material/core";

export const APP_DATE_FORMATS = {
  parse: {
    dateInput: 'DD-MM-YYYY',
  },
  display: {
    // Property in display section is the date format in which displays the date in input box.
    dateInput: 'date_format_for_input',
    // Property in display section is the date format in which calendar displays the month-year label.
    monthYearLabel: {year: 'numeric', month: 'short'},
    // Related to Accessibility (a11y)
    dateA11yLabel: {year: 'numeric', month: 'long', day: 'numeric'},
    monthYearA11yLabel: {year: 'numeric', month: 'long'},
  }
};

@Injectable()
export class AppDateAdapter extends NativeDateAdapter {
  override format(date: Date, displayFormat: Object): string {
    if (displayFormat === 'date_format_for_input') {
      return formatDate(date,'dd-MM-yyyy', this.locale);
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

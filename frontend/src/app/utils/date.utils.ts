export class DateUtil {
  /** Get date information in the specified format. */
  /*interface DateTimeFormatOptions {
      localeMatcher?: "best fit" | "lookup" | undefined;
      weekday?: "long" | "short" | "narrow" | undefined;
      era?: "long" | "short" | "narrow" | undefined;
      year?: "numeric" | "2-digit" | undefined;
      month?: "numeric" | "2-digit" | "long" | "short" | "narrow" | undefined;
      day?: "numeric" | "2-digit" | undefined;
      hour?: "numeric" | "2-digit" | undefined;
      minute?: "numeric" | "2-digit" | undefined;
      second?: "numeric" | "2-digit" | undefined;
      timeZoneName?: "short" | "long" | "shortOffset" | "longOffset" | "shortGeneric" | "longGeneric" | undefined;
      formatMatcher?: "best fit" | "basic" | undefined;
      hour12?: boolean | undefined;
      timeZone?: string | undefined;
  }*/
  public static formatDateTime(d: Date, options: Intl.DateTimeFormatOptions | undefined, locales?: string | string[] | undefined): string {
    return new Intl.DateTimeFormat(locales || 'default', options).format(d);
  }

  /** Add "delta" years for the specified date. */
  public static addYear(d: Date, delta: number = 1): Date {
    return new Date(d.getFullYear() + delta, d.getMonth(), d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds(), d.getMilliseconds());
  }
  /** Add "delta" months for the specified date. */
  public static addMonth(d: Date, delta: number = 1): Date {
    return new Date(d.getFullYear(), d.getMonth() + delta, d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds(), d.getMilliseconds());
  }
  /** Add "delta" days for the specified date. */
  public static addDay(d: Date, delta: number = 1): Date {
    return new Date(d.getFullYear(), d.getMonth(), d.getDate() + delta, d.getHours(), d.getMinutes(), d.getSeconds(), d.getMilliseconds());
  }
  /** Month in JavaScript is 0-indexed (January is 0, February is 1, etc), but by using 0 as the day
   *  it will give us the last day of the prior month. So passing in 1 as the month number will return
   *  the last day of January, not February.
   */
  public static daysInMonth(date: Date): number {
    return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
  }

  public static dateFirstDayOfMonth(d: Date): Date {
    return new Date(d.getFullYear(), d.getMonth(), 1, d.getHours(), d.getMinutes(), d.getSeconds(), d.getMilliseconds());
  }
  public static dateLastDayOfMonth(d: Date): Date {
    const day = DateUtil.daysInMonth(d);
    return new Date(d.getFullYear(), d.getMonth(), day, d.getHours(), d.getMinutes(), d.getSeconds(), d.getMilliseconds());
  }
}
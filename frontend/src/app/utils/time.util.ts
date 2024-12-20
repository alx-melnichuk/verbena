export class TimeUtil {
  /** Parsing 'hh:mm' or 'hh:mm:ss' into a structure with parameters. */
  public static parseTime(value: string | null | undefined): { hours: number, minutes: number, seconds: number } | null {
    let hours: number = -1;
    let minutes: number = -1;
    let seconds: number = 0;
    if (!!value && value.length > 4 && value.slice(2, 3) == ':') {
      hours = parseInt(value.slice(0, 2), 10);
      minutes = parseInt(value.slice(3, 5), 10);
    }
    if (!!value && value.length > 7 && value.slice(5, 6) == ':') {
      seconds = parseInt(value.slice(6, 8), 10);
    }
    return !isNaN(hours) && !isNaN(minutes) && !isNaN(seconds)
      && hours > -1 && minutes > -1 && seconds > -1 ? { hours, minutes, seconds } : null;
  }
  public static parseTimeHHMM(value: string | null | undefined): { hours: number, minutes: number } {
    let hours: number = 0;
    let minutes: number = 0;
    if (!!value && value.length > 4 && value.slice(2, 3) == ':') {
        const hoursStr = value.slice(0, 2);
        hours = parseInt(hoursStr, 10);
        const minutesStr = value.slice(3, 6);
        minutes = parseInt(minutesStr, 10);
    }
    return { hours, minutes };
  }
  // Time addition method.
  //   value: 'hh:mm' or 'hh:mm:ss'
  public static addTime(
    value: string | null | undefined, addHours: number, addMinutes: number, addSeconds: number
  ): { hours: number, minutes: number, seconds: number } | null {
    let hours: number = -1;
    let minutes: number = -1;
    let seconds: number = -1;
    const dateValue = TimeUtil.parseTime(value); // { hours: number, minutes: number, seconds: number } | null
    if (!!dateValue) {
      const date1 = new Date(Date.now());
      date1.setHours(dateValue.hours + addHours, dateValue.minutes + addMinutes, dateValue.seconds + addSeconds, 0);
      hours = date1.getHours();
      minutes = date1.getMinutes();
      seconds = date1.getSeconds();
    }
    return (!!dateValue ? { hours, minutes, seconds } : null);
  }
}
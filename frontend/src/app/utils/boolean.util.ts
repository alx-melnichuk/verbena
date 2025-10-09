export class BooleanUtil {
  public static init(value: string | boolean | null | undefined): boolean | null {
    return ['', 'true'].indexOf('' + value) > -1 ? true : '' + value === 'false' ? false : null;
  }
}

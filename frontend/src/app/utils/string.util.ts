export class StringUtil {
    public static maxLength = 3;
    /** Capitalize only the first letter. */
    public static capitalizeOnlyFirstLetter(val: string | null | undefined): string | null {
        return !!val ? val.split("_").map((s) => s.charAt(0).toLocaleUpperCase()).join("") : null;
    }
}
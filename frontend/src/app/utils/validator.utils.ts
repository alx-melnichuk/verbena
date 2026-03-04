import { ValidatorFn, Validators } from "@angular/forms";

export class ValidatorUtils {
    // Create an array of check rules based on the specified parameters.
    public static prepare(params: { [key: string]: null | undefined | number | string | boolean; }): ValidatorFn[] {
        const resultValidator: ValidatorFn[] = [];
        if (typeof params["min"] == "number" && params["min"] > -1) {
            resultValidator.push(Validators.min(params["min"]));
        }
        if (typeof params["max"] == "number" && params["max"] > -1) {
            resultValidator.push(Validators.max(params["max"]));
        }
        if (!!params["required"]) {
            resultValidator.push(Validators.required);
        }
        if (!!params["email"]) {
            resultValidator.push(Validators.email);
        }
        const minLength = typeof params["minLength"] == "number" ? params["minLength"] : -1;
        if (minLength > -1) {
            resultValidator.push(Validators.minLength(minLength));
        }
        const maxLength = typeof params["maxLength"] == "number" ? params["maxLength"] : -1;
        if (maxLength > -1) {
            resultValidator.push(Validators.maxLength(maxLength));
        }
        if (typeof params["pattern"] == "string" && !!params["pattern"]) {
            resultValidator.push(Validators.pattern(params["pattern"]));
        }
        return resultValidator;
    }
}
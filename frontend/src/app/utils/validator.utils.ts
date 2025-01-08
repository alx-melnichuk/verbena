import { ValidationErrors, ValidatorFn, Validators } from '@angular/forms';

export class ValidatorUtils {
  // Create an array of check rules based on the specified parameters.
  public static prepare(params: { [key: string]: any; }): ValidatorFn[] {
    const resultValidator: ValidatorFn[] = [];
    if (params["min"] > -1) {
      resultValidator.push(Validators.min(params["min"]));
    }
    if (params["max"] > -1) {
      resultValidator.push(Validators.max(params["max"]));
    }
    if (!!params["required"]) {
      resultValidator.push(Validators.required);
    }
    if (!!params["email"]) {
      resultValidator.push(Validators.email);
    }
    const minLength = 0 + (params["minLength"] || -1);
    if (minLength > -1) {
      resultValidator.push(Validators.minLength(minLength));
    }
    const maxLength = 0 + (params["maxLength"] || -1);
    if (maxLength > -1) {
      resultValidator.push(Validators.maxLength(maxLength));
    }
    if (!!params["pattern"]) {
      resultValidator.push(Validators.pattern(params["pattern"]));
    }
    return resultValidator;
  }
  // Get an error template based on the specified parameters.
  public static getErrorMsg(errors: ValidationErrors | null, name: string): string {
    let result: string = '';
    const errorList: string[] = Object.keys(errors || {});
    for (let index = 0; index < errorList.length && !result; index++) {
      const error: string = errorList[index];
      result = !result && 'required' === error ? `Validation.${name}:required` : result;
      result = !result && 'minlength' === error ? `Validation.${name}:min_length` : result;
      result = !result && 'maxlength' === error ? `Validation.${name}:max_length` : result;
      result = !result && 'pattern' === error ? `Validation.${name}:regex` : result;
      result = !result && 'email' === error ? `Validation.${name}:email_type` : result;
    }
    return result;
  }
}
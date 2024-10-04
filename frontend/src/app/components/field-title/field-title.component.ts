import {
  ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
  ValidationErrors, Validator, ValidatorFn,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';
import { ValidatorUtils } from 'src/app/utils/validator.utils';
  
export const TITLE = 'nickname';
export const TITLE_MIN_LENGTH = 2;
export const TITLE_MAX_LENGTH = 255;
export const CUSTOM_ERROR = 'customError';
  
@Component({
  selector: 'app-field-title',
  exportAs: 'appFieldTitle',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule],
  templateUrl: './field-title.component.html',
  styleUrls: ['./field-title.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldTitleComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldTitleComponent), multi: true },
  ],
})
export class FieldTitleComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public gist: string = TITLE;
  @Input()
  public hint: string = '';
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public isSpellcheck: boolean = false;
  @Input()
  public label: string = 'field-title.label';
  @Input()
  public maxLen: number = TITLE_MAX_LENGTH;
  @Input()
  public minLen: number = TITLE_MIN_LENGTH;
  @Input()
  public type: string = "text";
  @Input()
  public errorMsg: string | null | undefined;

  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ nickname: this.formControl });

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen'] || !!changes['type']) {
      this.prepareFormGroup();
    }
    if (!!changes['isDisabled']) {
      this.setDisabledState(this.isDisabled);
    }
    if (!!changes['errorMsg']) {
      this.prepareErrMsg(this.errorMsg);
    }
  }

  // ** ControlValueAccessor - start **

  public onChange: (val: string) => void = () => {};
  public onTouched: () => void = () => {};

  public writeValue(value: any): void {
    this.formControl.setValue(value, { emitEvent: true });
  }

  public registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  public registerOnTouched(fn: any): void {
    this.onTouched = fn;
  }

  public setDisabledState(isDisabled: boolean): void {
    isDisabled ? this.formGroup.disable() : this.formGroup.enable();
  }

  // ** ControlValueAccessor - finish **

  // ** Validator - start **

  public validate(control: AbstractControl): ValidationErrors | null {
    return this.formControl.errors;
  }

  // ** Validator - finish **

  // ** Public API **

  public focus(): void {
    this.matInput?.focus();
  }

  public getErrorMsg(errors: ValidationErrors | null): string {
    return ValidatorUtils.getErrorMsg(errors, this.gist || TITLE);
  }

  public getFormControl(): FormControl {
    return this.formControl;
  }
  
  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const paramsObj = {
      ...(this.isRequired ? { "required": true } : {}),
      ...(this.minLen > 0 ? { "minLength": this.minLen } : {}),
      ...(this.maxLen > 0 ? { "maxLength": this.maxLen } : {}),
      ...(this.type == "email" ? { "email": true } : {})
    };
    const newValidator: ValidatorFn[] = ValidatorUtils.prepare(paramsObj);
    this.formControl.setValidators(newValidator);
    this.formControl.updateValueAndValidity();
  }

  private prepareErrMsg(errMsg: string | null | undefined): void {
    let result: ValidationErrors | null = null;
    const errorsObj = {...this.formControl.errors};
    if (!!errMsg) {
      result = {...errorsObj, ...{ [CUSTOM_ERROR]: true } };
    } else {
      const list = Object.keys(errorsObj);
      let res: ValidationErrors = {};
      for (let index = 0; index < list.length; index++) {
        const key = list[index];
        if (key !== CUSTOM_ERROR) {
          res[key] = errorsObj[key];
        }
      }
      result = (Object.keys(res).length > 0 ? res : null);
    }
    this.formControl.setErrors(result, { emitEvent: true });
  }
}

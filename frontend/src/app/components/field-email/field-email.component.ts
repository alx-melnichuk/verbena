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

export const EMAIL = "email";
export const EMAIL_MIN_LENGTH = 5;
// https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address
// What is the maximum length of a valid email address? 
// Answer: An email address must not exceed 254 characters.
export const EMAIL_MAX_LENGTH = 254;

@Component({
  selector: 'app-field-email',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule],
  templateUrl: './field-email.component.html',
  styleUrls: ['./field-email.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldEmailComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldEmailComponent), multi: true },
  ],
})
export class FieldEmailComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public gist: string = EMAIL;
  @Input()
  public hint: string = '';
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-email.label';
  @Input()
  public maxLen: number = EMAIL_MAX_LENGTH;
  @Input()
  public minLen: number = EMAIL_MIN_LENGTH;
  @Input()
  public pattern: string = "";
  @Input()
  public type: string = "email";

  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ email: this.formControl });
  public errMessage: string = '';

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen'] || !!changes['pattern'] || !!changes['type']) {
      this.prepareFormGroup();
    }
    if (!!changes['isDisabled']) {
      this.setDisabledState(this.isDisabled);
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
    return ValidatorUtils.getErrorMsg(errors, this.gist || EMAIL);
  }

  // ** Private API **

  private prepareFormGroup(): void {
    const paramsObj = {
      ...(this.isRequired ? { "required": true } : {}),
      ...(this.minLen > 0 ? { "minLength": this.minLen } : {}),
      ...(this.maxLen > 0 ? { "maxLength": this.maxLen } : {}),
      ...(this.pattern ? { "pattern": this.pattern } : {}),
      ...(this.type == "email" ? { "email": true } : {})
    };
    const newValidator: ValidatorFn[] = ValidatorUtils.prepare(paramsObj);
    this.formControl.setValidators(newValidator);
    this.formControl.updateValueAndValidity();
  
  }
}

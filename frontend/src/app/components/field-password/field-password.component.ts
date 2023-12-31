import {
  ChangeDetectionStrategy,
  Component,
  Input,
  OnChanges,
  SimpleChanges,
  ViewChild,
  ViewEncapsulation,
  forwardRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl,
  ControlValueAccessor,
  FormControl,
  FormGroup,
  NG_VALIDATORS,
  NG_VALUE_ACCESSOR,
  ReactiveFormsModule,
  ValidationErrors,
  Validator,
  ValidatorFn,
  Validators,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';

export const PASSWORD_MIN_LENGTH = 6;
export const PASSWORD_MAX_LENGTH = 64;
export const PASSWORD_MAX_PATTERN = '^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)[A-Za-z\\d\\W_]{6,}$';

@Component({
  selector: 'app-field-password',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule],
  templateUrl: './field-password.component.html',
  styleUrls: ['./field-password.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldPasswordComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldPasswordComponent), multi: true },
  ],
})
export class FieldPasswordComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-password.label';
  @Input()
  public minLen: number = PASSWORD_MIN_LENGTH;
  @Input()
  public maxLen: number = PASSWORD_MAX_LENGTH;
  @Input()
  public pattern: string = PASSWORD_MAX_PATTERN;
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public hint: string = '';

  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ password: this.formControl });
  public isShowPassword = false;
  public errMessage: string = '';

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen']) {
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
    if (isDisabled != this.formGroup.disabled) {
        if (isDisabled) {
          this.isShowPassword = false;
          this.formGroup.disable();
        } else {
          this.formGroup.enable();
        }
      }
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
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? 'Validation.password:required' : result;
      result = !result && 'minlength' === error ? 'Validation.password:min_length' : result;
      result = !result && 'maxlength' === error ? 'Validation.password:max_length' : result;
      result = !result && 'pattern' === error ? 'Validation.password:regex' : result;

    }
    return result;
  }

  public showPassword(isShowPassword: boolean): void {
    if (this.isShowPassword !== isShowPassword) {
      this.isShowPassword = isShowPassword;
    }
  }

  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const newValidator: ValidatorFn[] = [
      ...(this.isRequired ? [Validators.required] : []),
      ...(this.minLen > 0 ? [Validators.minLength(this.minLen)] : []),
      ...(this.maxLen > 0 ? [Validators.maxLength(this.maxLen)] : []),
    ];
    this.formControl.setValidators(newValidator);
  }
}

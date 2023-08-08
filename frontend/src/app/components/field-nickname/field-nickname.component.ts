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

export const NICKNAME_MIN_LENGTH = 3;
export const NICKNAME_MAX_LENGTH = 10; // 32
export const NICKNAME_PATTERN = '^[a-zA-Z0-9]+$';

@Component({
  selector: 'app-field-nickname',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule],
  templateUrl: './field-nickname.component.html',
  styleUrls: ['./field-nickname.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldNicknameComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldNicknameComponent), multi: true },
  ],
})
export class FieldNicknameComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-nickname.label';
  @Input()
  public minLen: number = NICKNAME_MIN_LENGTH;
  @Input()
  public maxLen: number = NICKNAME_MAX_LENGTH;
  @Input()
  public pattern: string = NICKNAME_PATTERN;
  @Input()
  public isDisabled: boolean = false;

  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ nickname: this.formControl });
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
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? 'error.required' : result;
      result = !result && 'pattern' === error ? 'field-nickname.err_pattern' : result;
      result = !result && 'minlength' === error ? 'field-nickname.err_minlength' : result;
      result = !result && 'maxlength' === error ? 'field-nickname.err_maxlength' : result;
    }
    return result;
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

import {
  ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, 
  ValidationErrors, Validator, ValidatorFn, Validators
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';

import { TimeUtil } from 'src/app/utils/time.util';

export const FT_DEFAULT_STEP = 60;
export const FT_LENGTH_MIN = 5;

@Component({
  selector: 'app-field-time',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule],
  templateUrl: './field-time.component.html',
  styleUrls: ['./field-time.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldTimeComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldTimeComponent), multi: true },
  ],
})
export class FieldTimeComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-time.label';
  @Input()
  public hint: string = '';
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public min: string | null = null;
  @Input()
  public max: string | null = null;
  @Input()
  public step: number = FT_DEFAULT_STEP;
  
  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ time: this.formControl });
  public errMessage: string = '';

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired'] || !!changes['min'] || !!changes['max']) {
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

  // ** Validator - start ** createFieldTimeMinValidator()

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
      result = !result && 'required' === error ? 'Validation.time:required' : result;
      result = !result && 'min' === error ? 'Validation.time:min' : result;
      result = !result && 'max' === error ? 'Validation.time:max' : result;
    }
    return result;
  }

  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const newValidator: ValidatorFn[] = [
      ...(this.isRequired ? [Validators.required] : []),
      ...((this.min || '').length >= FT_LENGTH_MIN ? [this.timeMinValidator] : []),
      ...((this.max || '').length >= FT_LENGTH_MIN ? [this.timeMaxValidator] : []),
    ];
    this.formControl.setValidators(newValidator);
  }

  private getSeconds(value: { hours: number, minutes: number, seconds: number }): number {
    return value.hours * 36060 + value.minutes * 60 + value.seconds;
  }

  private timeMinValidator: ValidatorFn = (): ValidationErrors | null => {
    const curr = TimeUtil.parseTime(this.formControl.value || '');
    const min = TimeUtil.parseTime(this.min || '');
    return !!curr && !!min && this.getSeconds(curr) < this.getSeconds(min) 
      ? { 'min': {'requiredMin': this.min, 'actual': this.formControl.value }} 
      : null;
  }
  private timeMaxValidator: ValidatorFn = (): ValidationErrors | null => {
    const curr = TimeUtil.parseTime(this.formControl.value || '');
    const max = TimeUtil.parseTime(this.max || '');
    return !!curr && !!max && this.getSeconds(max) < this.getSeconds(curr) 
      ? { 'max': {'requiredMax': this.max, 'actual': this.formControl.value }} 
      : null;
  }
}

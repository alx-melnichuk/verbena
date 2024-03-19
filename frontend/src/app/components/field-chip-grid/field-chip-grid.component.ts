import {
  ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
  ValidationErrors, Validator, ValidatorFn
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatChipInputEvent, MatChipsModule } from '@angular/material/chips';
import { TranslateModule } from '@ngx-translate/core';
import { ENTER } from '@angular/cdk/keycodes';

@Component({
  selector: 'app-field-chip-grid',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, MatChipsModule],
  templateUrl: './field-chip-grid.component.html',
  styleUrls: ['./field-chip-grid.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldChipGridComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldChipGridComponent), multi: true },
  ],
})
export class FieldChipGridComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public label: string = 'field-chip-grid.label';
  @Input()
  public hint: string = 'field-chip-grid.hint';
  @Input()
  public isDisabled: boolean | null = false;
  @Input()
  public isReadonly: boolean | null = false;
  @Input()
  public isRequired: boolean | null = false;
  @Input()
  public isRemovable: boolean | null = false;
  @Input()
  public minLength: number | null = null;
  @Input()
  public maxLength: number | null = null;
  @Input()
  public minQuantity: number | null = null;
  @Input()
  public maxQuantity: number | null = null;
  @Input()
  public separatorCodes: readonly number[] | ReadonlySet<number> = [ENTER];
  
  @ViewChild('chipInput', { read: HTMLInputElement, static: false }) 
  public chipInputElem: HTMLInputElement | undefined;

  public formControl: FormControl = new FormControl({ value: [], disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ form: this.formControl });

  readonly value: string[] = this.formControl.value.concat();

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired'] || !!changes['minLength'] || !!changes['maxLength']
      || !!changes['minQuantity'] || !!changes['maxQuantity']) {
      this.prepareFormGroup();
    }
    if (!!changes['isDisabled']) {
      this.setDisabledState(!!this.isDisabled);
    }
  }

  // ** ControlValueAccessor - start **

  public onChange: (val: string[] | null) => void = () => {};
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
    this.chipInputElem?.focus();
  }

  public getErrorMsg(errors: ValidationErrors | null): string {
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? 'Validation.tag:required' : result;
      result = !result && 'minQuantity' === error ? 'Validation.tag:min_quantity' : result;
      result = !result && 'maxQuantity' === error ? 'Validation.tag:max_quantity' : result;
      result = !result && 'minlength' === error ? 'Validation.tag:min_length' : result;
      result = !result && 'maxlength' === error ? 'Validation.tag:max_length' : result;
    }
    return result;
  }

  public chipRemove(chipValue: string, chipValues: string[] | null): void {
    const chipValueList: string[] = (chipValues || []).concat();
    const index = chipValueList.indexOf(chipValue);
    if (index >= 0) {
      chipValueList.splice(index, 1);
      this.updateValueAndValidity(chipValueList);
    }
  }

  public chipAdd(event: MatChipInputEvent, chipValues: string[] | null): void {
    const chipValue = (event.value || '').trim();
    const chipValueList: string[] = (chipValues || []).concat();
    // Check for duplicate
    if (chipValue.length > 0 && !chipValueList.includes(chipValue)) {
      chipValueList.push(chipValue);
      this.updateValueAndValidity(chipValueList);
    }
    // Reset the input value
    event.chipInput?.clear();
  }

  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const newValidator: ValidatorFn[] = [
      ...(!!this.isRequired ? [this.requiredValidator] : []),
      ...((this.minLength || 0) > 0 ? [this.minLengthValidator] : []),
      ...((this.maxLength || 0) > 0 ? [this.maxLengthValidator] : []),
      ...((this.minQuantity || 0) > 0 ? [this.minQuantityValidator] : []),
      ...((this.maxQuantity || 0) > 0 ? [this.maxQuantityValidator] : []),
    ];
    this.formControl.setValidators(newValidator);
  }
  private requiredValidator: ValidatorFn = (): ValidationErrors | null => {
    const curr: string[] | null = this.formControl.value;
    const length = (curr || []).length;
    return !!curr && length == 0 ? { 'required': true } : null;
  }
  private minQuantityValidator: ValidatorFn = (): ValidationErrors | null => {
    const curr: string[] | null = this.formControl.value;
    const length = (curr || []).length;
    const min = this.minQuantity || 0;
    const actualQuantity = length;
    return !!curr && length > 0 && length < min ? { 'minQuantity': { actualQuantity, "requiredQuantity": min }} : null;
  }
  private maxQuantityValidator: ValidatorFn = (): ValidationErrors | null => {
    const curr: string[] | null = this.formControl.value;
    const length = (curr || []).length;
    const max = this.maxQuantity || 0;
    const actualQuantity = length;
    return !!curr && length > 0 && length > max ? { 'maxQuantity': { actualQuantity, "requiredQuantity": max }} : null;
  }
  private minLengthValidator: ValidatorFn = (): ValidationErrors | null => {
    let result: ValidationErrors | null = null;
    if (this.minLength != null && this.minLength > 0) {
      const currValuesList: string[] = this.formControl.value || [];
      for (let index = 0; index < currValuesList.length && result == null; index++) {
        const item = currValuesList[index];
        result = item.length < this.minLength
          ? { 'minlength': { "requiredLength": this.minLength, "actualLength": item.length, "actualValue": item }}
          : result;
      }
    }
    return result;
  }
  private maxLengthValidator: ValidatorFn = (): ValidationErrors | null => {
    let result: ValidationErrors | null = null;
    if (this.maxLength != null && this.maxLength > 0) {
      const currValuesList: string[] = this.formControl.value || [];
      for (let index = 0; index < currValuesList.length && result == null; index++) {
        const item = currValuesList[index];
        result = item.length > this.maxLength
          ? { 'maxlength': { "requiredLength": this.maxLength, "actualLength": item.length, "actualValue": item }}
          : result;
      }
    }
    return result;
  }
  /** Update the data value and perform validation. */
  private updateValueAndValidity(value: string[] | null): void {
    this.formControl.setValue(value, { emitEvent: true });
    // Calling the validation method for the new value.
    this.onChange(value);
  }
}

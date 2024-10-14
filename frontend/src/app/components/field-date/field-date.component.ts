import {
  ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, 
  ValidationErrors, Validator, ValidatorFn, Validators
} from '@angular/forms';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';
  
@Component({
  selector: 'app-field-date',
  exportAs: 'appFieldDate',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, MatDatepickerModule],
  templateUrl: './field-date.component.html',
  styleUrls: ['./field-date.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldDateComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldDateComponent), multi: true },
  ],
})
export class FieldDateComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public hint: string = '';
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-date.label';
  @Input()
  public maxDate: Date | null | undefined;
  @Input()
  public minDate: Date | null | undefined;
  @Input()
  public errorMsg: string | null | undefined;

  @Output()
  readonly dateInput: EventEmitter<Date | null> = new EventEmitter();
  @Output()
  readonly dateChange: EventEmitter<Date | null> = new EventEmitter();

  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ date: this.formControl });

  constructor() {}

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired']) {
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
    return this.addDateStringToError(this.formControl.errors);
  }

  // ** Validator - finish **

  // ** Public API **

  public focus(): void {
    this.matInput?.focus();
  }

  public getErrorMsg(errors: ValidationErrors | null): string {
    let result: string = '';
    const errors2 = this.addDateStringToError(errors);
    const errorsList: string[] = errors2 != null ? Object.keys(errors2) : [];
    const idxRequired = errorsList.indexOf('required');
    const errorProps = (idxRequired > -1 ? errorsList.splice(idxRequired, 1) : errorsList);
    for (let index = 0; index < errorProps.length && !result; index++) {
      const error: string = errorProps[index];
      result = !result && 'matDatepickerParse' === error ? 'Validation.date:invalid_format' : result;
      result = !result && 'matDatepickerMin' === error ? 'Validation.date:minDate' : result;
      result = !result && 'matDatepickerMax' === error ? 'Validation.date:maxDate' : result;
    }
    if (!result && idxRequired > -1) {
      result = 'Validation.date:required';
    }
    return result;
  }

  public doDateInput(e: any): void {
    this.dateInput.emit(this.formControl.value);
  }
  
  public doDateChange(e: any): void {
    this.dateChange.emit(this.formControl.value);
    this.onChange(this.formControl.value);
  }

  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const newValidator: ValidatorFn[] = [
      ...(this.isRequired ? [Validators.required] : []),
    ];
    this.formControl.setValidators(newValidator);
  }
  private addDateStringToError(errors: ValidationErrors | null): ValidationErrors | null {
    if (errors != null) {
      const minDate = errors['matDatepickerMin'];
      if (minDate?.min != null && minDate?.min_s == null) {
        minDate['min_s'] = minDate.min.toISOString().slice(0,10);
      }
      if (minDate?.actual != null && minDate?.actual_s == null) {
        minDate['actual_s'] = minDate.actual.toISOString().slice(0,10);
      }
      const maxDate = errors['matDatepickerMax'];
      if (maxDate?.max != null && maxDate?.max_s == null) {
        maxDate['max_s'] = maxDate.max.toISOString().slice(0,10);
      }
      if (maxDate?.actual != null && maxDate?.actual_s == null) {
        maxDate['actual_s'] = maxDate.actual.toISOString().slice(0,10);
      }
    }
    return errors;
  }
}

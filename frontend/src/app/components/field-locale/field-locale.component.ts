import {
  ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, 
  ValidationErrors, Validator, ValidatorFn, Validators
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelect, MatSelectModule } from '@angular/material/select';
import { TranslateModule } from '@ngx-translate/core';

import { LOCALE_DE_DE, LOCALE_EN_US, LOCALE_NOTHING, LOCALE_UK } from 'src/app/common/constants';

@Component({
  selector: 'app-field-locale',
  exportAs: 'appFieldLocale',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, MatSelectModule],
  templateUrl: './field-locale.component.html',
  styleUrls: ['./field-locale.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldLocaleComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldLocaleComponent), multi: true },
  ],
})
export class FieldLocaleComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-locale.label';
  @Input()
  public hint: string = '';
  @Input()
  public isDisabled: boolean = false;

  @Output()
  readonly change: EventEmitter<string> = new EventEmitter();
  @Output()
  readonly openedChange: EventEmitter<boolean> = new EventEmitter();

  @ViewChild(MatSelect, { static: false })
  public matSelect: MatSelect | null = null;

  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ locale: this.formControl });
  public errMessage: string = '';

  public nothing = LOCALE_NOTHING;
  public localeList: string[] = ['', LOCALE_EN_US, LOCALE_DE_DE, LOCALE_UK];

  constructor() {
  }

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

  // ** Validator - start **

  public validate(control: AbstractControl): ValidationErrors | null {
    return this.formControl.errors;
  }

  // ** Validator - finish **

  // ** Public API **

  public focus(): void {
    this.matSelect?.focus();
  }

  public doOpenedChange(event: boolean): void {
    this.openedChange.emit(event);
  }

  public doSelectionChange(value: string): void {
    this.change.emit(value);
    this.onChange(this.formControl.value);
  }

  public getErrorMsg(errors: ValidationErrors | null): string {
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? 'Validation.locale:required' : result;
    }
    return result;
  }

  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const newValidator: ValidatorFn[] = [
      ...(this.isRequired ? [Validators.required] : []),
    ];
    this.formControl.setValidators(newValidator);
  }
}

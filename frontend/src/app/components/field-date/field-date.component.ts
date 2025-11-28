import {
    ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation,
    forwardRef, inject
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
    AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
    ValidationErrors, Validator, ValidatorFn, Validators
} from '@angular/forms';
import { DateAdapter } from '@angular/material/core';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslatePipe } from '@ngx-translate/core';

import { CalendarHeaderComponent } from '../calendar-header/calendar-header.component';

@Component({
    selector: 'app-field-date',
    exportAs: 'appFieldDate',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, MatDatepickerModule, TranslatePipe],
    templateUrl: './field-date.component.html',
    styleUrl: './field-date.component.scss',
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
    public locale: string | null = null;
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

    readonly calendarHeader = CalendarHeaderComponent;
    private readonly dateAdapter: DateAdapter<Date> = inject(DateAdapter);
    private localeError: string | null = null;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired']) {
            this.prepareFormGroup();
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(this.isDisabled);
        }
        if (!!changes['locale'] && !!this.formControl.errors) {
            // In the case where an error is already displayed and the locale changes, it is necessary 
            // to update the value of the parameter of the "translate" directive (this.formControl.errors).
            // This needs to be done to display the error in the date format for the new locale.
            this.validate(this.formControl);
        }
    }

    // ** ControlValueAccessor - start **

    public onChange: (val: string) => void = () => { };
    public onTouched: () => void = () => { };

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
        if (this.formControl.errors != null) {
            const oldErrors = this.formControl.errors;
            // Add extended tags with date format for the current locale. (min_s, max_s, actual_s)
            const errors = this.addExtendTags({ ...this.formControl.errors }, this.locale);

            // When called, the "translate" directive compares its parameter with the previous value.
            // And if the new value of the parameter does not differ from the old one, then the directive is not rendered.
            // In this case, creating a new object for the new value of the parameter is not enough.
            // Since the parameter value object is compared with the old value for each property of the object.
            // For this reason, an additional property is added to the parameter to distinguish the old value.
            // This is very important in the case where an error is already displayed and the locale changes. 
            // In this case, it is necessary to update the value of the parameter of the "translate" directive
            // (this.formControl.errors).

            // Create a new error object.
            this.formControl.setErrors(errors);
            // Additional fields to differentiate from the old version.
            oldErrors[`difference~$#`] = 1;
        }
        return this.formControl.errors;
    }

    // ** Validator - finish **

    // ** Public API **

    public focus(): void {
        this.matInput?.focus();
    }

    public getErrorMsg(errors: ValidationErrors | null): string {
        let result: string = '';
        const errorList: string[] = Object.keys(errors || {});
        const idxRequired = errorList.indexOf('required');
        const errorList2 = (idxRequired > -1 ? errorList.splice(idxRequired, 1) : errorList);
        for (let index = 0; index < errorList2.length && !result; index++) {
            const error: string = errorList2[index];
            result = !result && 'matDatepickerParse' === error ? 'ExpectationFailed.date:invalid_format' : result;
            result = !result && 'matDatepickerMin' === error ? 'ExpectationFailed.date:minDate' : result;
            result = !result && 'matDatepickerMax' === error ? 'ExpectationFailed.date:maxDate' : result;
        }
        if (!result && idxRequired > -1) {
            result = 'ExpectationFailed.date:required';
        }
        return result;
    }

    public doDateInput(e: any): void {
        // Add extended tags with date format for the current locale. (min_s, max_s, actual_s)
        this.validate(this.formControl);
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
    // Add extended tags. (min_s, max_s, actual_s)
    private checkDateAndMapStr(error_item: ValidationErrors | null, key: string, isLocaleShift: boolean): ValidationErrors | null {
        const key2 = key + '_s';
        if (error_item != null && !!key && error_item[key] != null && (error_item[key2] == null || isLocaleShift)) {
            error_item[key2] = this.dateAdapter.format(error_item[key], null);
        }
        return error_item;
    }
    /** Add extended tags with date format for the current locale. (min_s, max_s, actual_s) */
    private addExtendTags(errors: ValidationErrors | null, locale: string | null): ValidationErrors | null {
        if (errors != null) {
            const isLocaleShift = this.localeError != locale;

            const minDate = errors['matDatepickerMin'];
            this.checkDateAndMapStr(minDate, 'min', isLocaleShift);
            this.checkDateAndMapStr(minDate, 'actual', isLocaleShift);

            const maxDate = errors['matDatepickerMax'];
            this.checkDateAndMapStr(maxDate, 'max', isLocaleShift);
            this.checkDateAndMapStr(maxDate, 'actual', isLocaleShift);

            if (isLocaleShift) {
                this.localeError = locale;
            }
        }
        return errors;
    }
}

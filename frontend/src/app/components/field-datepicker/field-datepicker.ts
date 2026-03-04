import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, ElementRef, EventEmitter, forwardRef, inject, Input, OnChanges, Output, SimpleChanges,
    ViewChild, ViewEncapsulation,
} from "@angular/core";
import {
    ReactiveFormsModule, NG_VALUE_ACCESSOR, NG_VALIDATORS, ControlValueAccessor, Validator, FormControl, FormGroup, AbstractControl, ValidationErrors, ValidatorFn, Validators
} from "@angular/forms";
// import { DateAdapter, MatDateFormats, MAT_DATE_FORMATS } from "@angular/material/core";
import { MatDatepickerInputEvent, MatDatepickerModule } from "@angular/material/datepicker";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule, MatInput } from "@angular/material/input";
import { TranslatePipe } from "@ngx-translate/core";
import { CalendarHeader } from "./calendar-header/calendar-header";
import { DatepickerExtTags } from "./datepicker-ext-tags";
import { DateUtil } from "../../utils/date.utils";

export const CUSTOM_ERROR = "customError";
export const DATEPICKER = "datepicker";

@Component({
    selector: "app-field-datepicker",
    exportAs: "appFieldDate",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatFormFieldModule, MatInputModule, MatDatepickerModule, TranslatePipe, CalendarHeader, DatepickerExtTags],
    templateUrl: "./field-datepicker.html",
    styleUrl: "./field-datepicker.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldDatepicker), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldDatepicker), multi: true },
    ],
})
export class FieldDatepicker implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string | null | undefined;
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public isReadOnly: boolean | null | undefined;
    @Input()
    public isRequired: boolean | null | undefined;
    @Input()
    public kind: string = DATEPICKER;
    @Input()
    public label: string | null | undefined;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public maxDate: Date | null | undefined;
    @Input()
    public minDate: Date | null | undefined;

    @Output()
    readonly dateInput: EventEmitter<Date | null> = new EventEmitter();
    @Output()
    readonly dateChange: EventEmitter<Date | null> = new EventEmitter();

    // private dateAdapter: DateAdapter<Date> = inject<DateAdapter<Date>>(DateAdapter, { optional: true })!;
    // private dateFormats: MatDateFormats = inject<MatDateFormats>(MAT_DATE_FORMATS, { optional: true })!;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ date: this.formControl });

    readonly calendarHeaderComp = CalendarHeader;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isRequired"]) {
            this.prepareFormGroup(this.isRequired || null);
        }
        if (!!changes["isDisabled"]) {
            this.setDisabledState(!!this.isDisabled);
        }
        if (!!changes["errorMsg"]) {
            this.formControl.updateValueAndValidity();
            this.onChange?.(this.formControl.value);
        }
        if (!!changes["locale"] && !!this.formControl.errors) {
            // In the case where an error is already displayed and the locale changes, it is necessary 
            // to update the value of the parameter of the "translate" directive (this.formControl.errors).
            // This needs to be done to display the error in the date format for the new locale.
            this.validate(this.formControl);
        }
    }

    // ** ControlValueAccessor - start **

    public onChange: ((value: any) => void) | undefined;
    public onTouched: (() => void) | undefined;
    // Writes a new value to the element.
    public writeValue(value: any): void {
        this.formControl.setValue(value, { emitEvent: true });
    }
    // Registers a callback function that is called when the control"s value changes in the UI.
    public registerOnChange(fn: (value: any) => void): void {
        this.onChange = fn;
    }
    // Registers a callback function that is called by the forms API on initialization to update the form model on blur.
    public registerOnTouched(fn: () => void): void {
        this.onTouched = fn;
    }
    // Function that is called by the forms API when the control status changes to `enabled` or from "disabled".
    public setDisabledState(isDisabled: boolean): void {
        if (isDisabled != this.formGroup.disabled) {
            isDisabled ? this.formGroup.disable() : this.formGroup.enable();
        }
    }

    // ** ControlValueAccessor - finish **

    // ** Validator - start ** createFieldTimeMinValidator()
    // Method that performs synchronous validation against the provided control.
    public validate(control: AbstractControl): ValidationErrors | null {
        return this.formControl.errors;
    }

    // ** Validator - finish **

    // ** Public API **

    public focus(): void {
        this.matInput?.focus();
    }
    public getErrorMsg(errors: ValidationErrors | null): string {
        const key = Object.keys(errors || {})[0];
        return !!key ? `417.field-${this.kind || DATEPICKER}:${key}` : "";
    }
    public getErrorObj(errors: ValidationErrors | null, value: string | null | undefined): ValidationErrors {
        // if (!!errors) {
        //     // Add extended tags with date format for the current locale. (min_s, max_s, actual_s)
        //     this.checkKeyAndAddExtendedTags(errors);
        // }
        return { ...errors, ...{ [`field-${this.kind || DATEPICKER}`]: value } };
    }
    public getFormControl(): FormControl {
        return this.formControl;
    }
    public markAsTouched(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsTouched(opts);
    }
    public markAllAsTouched(opts?: { emitEvent?: boolean; }): void {
        this.formControl.markAllAsTouched(opts);
    }
    public markAsUntouched(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsUntouched(opts);
    }
    public markAsDirty(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsDirty(opts);
    }
    public markAsPristine(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsPristine(opts);
    }
    public markAsPending(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsPending(opts);
    }

    public doDateInput(event: MatDatepickerInputEvent<Date>): void {
        // Add extended tags with date format for the current locale. (min_s, max_s, actual_s)
        this.validate(this.formControl);
        this.dateInput.emit(this.formControl.value);
    }

    public doDateChange(event: MatDatepickerInputEvent<Date>): void {
        this.onChange?.(this.formControl.value);
        this.dateChange.emit(this.formControl.value);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        return !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
    };

    private prepareFormGroup(isRequired: boolean | null): void {
        this.formControl.clearValidators();
        const newValidator: ValidatorFn[] = [
            ...(isRequired ? [Validators.required] : []),
        ];
        this.formControl.setValidators([...newValidator, this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }

    /** Add extended tags with date format for the current locale. (min_s, max_s, actual_s) */
    /*private checkKeyAndAddExtendedTags(errors: ValidationErrors | null): ValidationErrors | null {
        if (errors != null) {
            for (const nameKey of ["matDatepickerMin", "matDatepickerMax"]) {
                const error_item = errors[nameKey] || {};
                for (const key of Object.keys(error_item)) {
                    if (error_item[key] instanceof Date) {
                        error_item[key + "_s"] = this.dateAdapter.format(error_item[key], this.dateFormats.display.dateInput);
                    }
                }
            }
        }
        return errors;
    }*/
}

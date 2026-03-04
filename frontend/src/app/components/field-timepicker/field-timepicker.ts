import { CommonModule } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, forwardRef, OnChanges, Input, inject, ViewChild, SimpleChanges,
} from "@angular/core";
import {
    ReactiveFormsModule, NG_VALUE_ACCESSOR, NG_VALIDATORS, ControlValueAccessor, Validator, FormControl, FormGroup, AbstractControl, ValidationErrors, ValidatorFn, Validators
} from "@angular/forms";
// import { DateAdapter, MatDateFormats, MAT_DATE_FORMATS } from "@angular/material/core";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule, MatInput } from "@angular/material/input";
import { MatTimepickerModule, MatTimepicker } from "@angular/material/timepicker";
import { TranslatePipe } from "@ngx-translate/core";
import { TimepickerExtTags } from "./timepicker-ext-tags";

export const CUSTOM_ERROR = "customError";
export const TIMEPICKER = "timepicker";
export const FTP_LENGTH_MIN = 5;
export const FTP_TIME_REGEX = "^([01][0-9]|2[0-3]):[0-5][0-9]$";

@Component({
    selector: "app-field-timepicker",
    exportAs: "appFieldTimepicker",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatFormFieldModule, MatInputModule, MatTimepickerModule, TranslatePipe, TimepickerExtTags],
    templateUrl: "./field-timepicker.html",
    styleUrl: "./field-timepicker.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldTimepicker), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldTimepicker), multi: true },
    ],
})
export class FieldTimepicker implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string | null | undefined;
    // The interval between each option. Can be either a number of seconds (e.g. 90)
    // or a number with units (e.g. 900s, 15m, 0.25h - 15 minutes).
    @Input()
    public interval: number | string | null | undefined;
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public isReadOnly: boolean | null | undefined;
    @Input()
    public isRequired: boolean | null | undefined;
    @Input()
    public kind: string = TIMEPICKER;
    @Input()
    public label: string | null | undefined;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public maxTime: string | null | undefined; // `hh:mm` ^([01][0-9]|2[0-3]):[0-5][0-9]$
    @Input()
    public minTime: string | null | undefined; // `hh:mm` ^([01][0-9]|2[0-3]):[0-5][0-9]$

    // private dateAdapter: DateAdapter<Date> = inject<DateAdapter<Date>>(DateAdapter, { optional: true })!;
    // private dateFormats: MatDateFormats = inject<MatDateFormats>(MAT_DATE_FORMATS, { optional: true })!;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    @ViewChild(MatTimepicker, { static: false })
    public matTimepicker: MatTimepicker<Date> | null = null;

    public formControl: FormControl = new FormControl<Date | null>({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ time: this.formControl });

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
        return !!key ? `417.field-${this.kind || TIMEPICKER}:${key}` : "";
    }
    public getErrorObj(errors: ValidationErrors | null, value: string | null | undefined): ValidationErrors {
        // if (!!errors) {
        //     // Add extended tags with date format for the current locale. (min_s, max_s, actual_s)
        //     this.checkKeyAndAddExtendedTags(errors);
        // }
        return { ...errors, ...{ [`field-${this.kind || TIMEPICKER}`]: value } };
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

    public handleClick(): void {
        if (this.matTimepicker?.isOpen) {
            this.matTimepicker.close();
        }
    };

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
            for (const nameKey of ["matTimepickerMin", "matTimepickerMax"]) {
                const error_item = errors[nameKey] || {};
                for (const key of Object.keys(error_item)) {
                    if (error_item[key] instanceof Date) {
                        error_item[key + "_s"] = this.dateAdapter.format(error_item[key], this.dateFormats.display.timeInput);
                    }
                }
            }
        }
        return errors;
    }*/
}

import { Directive, forwardRef, Input, inject, OnChanges, SimpleChanges } from "@angular/core";
import { NG_VALIDATORS, Validator, AbstractControl, ValidationErrors } from "@angular/forms";
import { DateAdapter, MatDateFormats, MAT_DATE_FORMATS } from "@angular/material/core";

@Directive({
    selector: "input[matDatepicker][appDatepickerExtTags]",
    standalone: true,
    providers: [{ provide: NG_VALIDATORS, useExisting: forwardRef(() => DatepickerExtTags), multi: true }],
})
export class DatepickerExtTags implements OnChanges, Validator {
    @Input({ alias: "appDatepickerExtTagsMin" })
    public minTime: Date | null | undefined;
    @Input({ alias: "appDatepickerExtTagsMax" })
    public maxTime: Date | null | undefined;
    @Input({ alias: "appDatepickerExtTagsLocale" })
    public locale: string | null | undefined;

    private dateAdapter: DateAdapter<Date> = inject<DateAdapter<Date>>(DateAdapter, { optional: true })!;
    private dateFormats: MatDateFormats = inject<MatDateFormats>(MAT_DATE_FORMATS, { optional: true })!;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["locale"]) {
            this.onValidatorOnChange?.();
        }
    }

    // ** ControlValueAccessor - finish **

    // ** Validator - start ** createFieldTimeMinValidator()
    // Method that performs synchronous validation against the provided control.
    public validate(control: AbstractControl): ValidationErrors | null {
        let result: ValidationErrors = {};
        const value = this.dateAdapter.getValidDateOrNull(this.dateAdapter.deserialize(control.value));

        const resultMin = !this.minTime || !value || this.dateAdapter.compareDate(this.minTime, value) <= 0
            ? {}
            : {
                "appDatepickerExtTagsMin": {
                    "min": this.dateAdapter.format(this.minTime, this.dateFormats.display.dateInput),
                    "actual": this.dateAdapter.format(value, this.dateFormats.display.dateInput),
                }
            };
        result = { ...result, ...resultMin };

        const resultMax = !this.maxTime || !value || this.dateAdapter.compareDate(this.maxTime, value) >= 0
            ? {}
            : {
                "appDatepickerExtTagsMax": {
                    "max": this.dateAdapter.format(this.maxTime, this.dateFormats.display.dateInput),
                    "actual": this.dateAdapter.format(value, this.dateFormats.display.dateInput),
                }
            };
        result = { ...result, ...resultMax };

        return Object.keys(result).length === 0 ? null : result;
    }

    public onValidatorOnChange: (() => void) | undefined;
    // Registers a callback function to call when the validator inputs change.
    public registerOnValidatorChange(fn: () => void): void {
        this.onValidatorOnChange = fn;
    }

    // ** Validator - finish **

}

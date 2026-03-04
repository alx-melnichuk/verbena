import { Directive, forwardRef, OnChanges, Input, inject, SimpleChanges } from "@angular/core";
import { NG_VALIDATORS, Validator, AbstractControl, ValidationErrors } from "@angular/forms";
import { DateAdapter, MatDateFormats, MAT_DATE_FORMATS } from "@angular/material/core";

@Directive({
    selector: "input[matTimepicker][appTimepickerExtTags]",
    standalone: true,
    providers: [{ provide: NG_VALIDATORS, useExisting: forwardRef(() => TimepickerExtTags), multi: true }],
})
export class TimepickerExtTags implements OnChanges, Validator {
    @Input({ alias: "appTimepickerExtTagsMin" })
    public minTime: string | null | undefined;
    @Input({ alias: "appTimepickerExtTagsMax" })
    public maxTime: string | null | undefined;
    @Input({ alias: "appTimepickerExtTagsLocale" })
    public locale: string | null | undefined;

    private dateAdapter: DateAdapter<Date> = inject<DateAdapter<Date>>(DateAdapter, { optional: true })!;
    private dateFormats: MatDateFormats = inject<MatDateFormats>(MAT_DATE_FORMATS, { optional: true })!;

    private min: Date | null = null;
    private max: Date | null = null;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["minTime"]) {
            this.min = this.transformDateInput(this.minTime);
        }
        if (!!changes["maxTime"]) {
            this.max = this.transformDateInput(this.maxTime);
        }
        if (!!changes["locale"]) {
            this.onValidatorOnChange?.();
        }
    }

    // ** Validator - start ** createFieldTimeMinValidator()
    // Method that performs synchronous validation against the provided control.
    public validate(control: AbstractControl): ValidationErrors | null {
        let result: ValidationErrors = {};
        const value = this.dateAdapter.getValidDateOrNull(this.dateAdapter.deserialize(control.value));
        const resultMin = !this.min || !value || this.dateAdapter.compareTime(this.min, value) <= 0
            ? {}
            : {
                "appTimepickerExtTagsMin": {
                    "min": this.dateAdapter.format(this.min, this.dateFormats.display.timeInput),
                    "actual": this.dateAdapter.format(value, this.dateFormats.display.timeInput),
                },
            };
        result = { ...result, ...resultMin };

        const resultMax = !this.max || !value || this.dateAdapter.compareTime(this.max, value) >= 0
            ? {}
            : {
                "appTimepickerExtTagsMax": {
                    "max": this.dateAdapter.format(this.max, this.dateFormats.display.timeInput),
                    "actual": this.dateAdapter.format(value, this.dateFormats.display.timeInput),
                },
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

    public transformDateInput(value: unknown): Date | null {
        const date = typeof value === "string"
            ? this.dateAdapter.parseTime(value, this.dateFormats.parse.timeInput)
            : this.dateAdapter.deserialize(value);
        const res = date && this.dateAdapter.isValid(date) ? (date as Date) : null;

        return res;
    }
}

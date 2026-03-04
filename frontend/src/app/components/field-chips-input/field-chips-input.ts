import { ENTER } from "@angular/cdk/keycodes";
import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, EventEmitter, forwardRef, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation } from "@angular/core";
import { AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, ValidationErrors, Validator, ValidatorFn } from "@angular/forms";
import { MatChipInputEvent, MatChipsModule } from "@angular/material/chips";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { TranslatePipe } from "@ngx-translate/core";

export const CHIPS_INPUT = "chips_input";
export const CUSTOM_ERROR = "customError";

@Component({
    selector: "app-field-chips-input",
    exportAs: "appFieldChipsInput",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatFormFieldModule,
        MatInputModule, MatChipsModule, TranslatePipe],
    templateUrl: "./field-chips-input.html",
    styleUrl: "./field-chips-input.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldChipsInput), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldChipsInput), multi: true },
    ],
})
export class FieldChipsInput implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string | null | undefined;
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public isReadOnly: boolean | null | undefined;
    @Input()
    public isRemovable: boolean | null | undefined;
    @Input()
    public isRequired: boolean | null | undefined;
    @Input()
    public kind: string = CHIPS_INPUT;
    @Input()
    public label: string | null | undefined;
    @Input()
    public maxLen: number | null | undefined;
    @Input()
    public minLen: number | null | undefined;
    @Input()
    public minAmount: number | null | undefined;
    @Input()
    public maxAmount: number | null | undefined;
    @Input()
    public separatorCodes: readonly number[] | ReadonlySet<number> = [ENTER];

    @Output()
    readonly removedChip: EventEmitter<string> = new EventEmitter();

    @ViewChild("chipInput", { read: HTMLInputElement, static: false })
    public chipInputElem: HTMLInputElement | undefined;

    public formControl: FormControl = new FormControl({ value: [], disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ form: this.formControl });

    readonly value: string[] = this.formControl.value.concat();

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isRequired"] || !!changes["maxLen"] || !!changes["minLen"] || !!changes["maxAmount"] || !!changes["minAmount"]) {
            this.prepareFormGroup(this.isRequired || null, this.maxLen || null, this.minLen || null
                , this.maxAmount || null, this.minAmount || null);
        }
        if (!!changes["isDisabled"]) {
            this.setDisabledState(!!this.isDisabled);
        }
        if (!!changes["errorMsg"]) {
            this.formControl.updateValueAndValidity();
            this.onChange(this.formControl.value);
        }
    }

    // ** ControlValueAccessor - start **

    public onChange: (val: string[] | null) => void = () => { };
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
        if (isDisabled != this.formGroup.disabled) {
            if (isDisabled) {
                this.formGroup.disable();
            } else {
                this.formGroup.enable();
            }
        }
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
        const key = Object.keys(errors || {})[0];
        return !!key ? `417.field-${this.kind || CHIPS_INPUT}:${key}` : "";
    }
    public getErrorObj(errors: ValidationErrors | null, value: string | null | undefined): ValidationErrors {
        return { ...errors, ...{ [`field-${this.kind || CHIPS_INPUT}`]: value } };
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

    public chipRemove(chipValue: string, chipValues: string[] | null): void {
        const chipValueList: string[] = (chipValues || []).concat();
        const index = chipValueList.indexOf(chipValue);
        if (index >= 0) {
            const chipValue = chipValueList.splice(index, 1)[0];
            this.updateValueAndValidity(chipValueList);
            if (!!this.isRemovable) {
                this.removedChip.emit(chipValue)
            }
        }
    }

    public chipAdd(event: MatChipInputEvent, chipValues: string[] | null): void {
        const chipValue = (event.value || "").trim();
        const chipValueList: string[] = (chipValues || []).concat();
        // Check for duplicate
        if (chipValue.length > 0 && !chipValueList.includes(chipValue)) {
            chipValueList.push(chipValue);
            this.updateValueAndValidity(chipValueList);
        }
        // Reset the input value
        event.chipInput!.clear();
    }

    public checkChipMinLen(chip: string | null | undefined, minLen: number | null | undefined): boolean {
        return (chip?.length || 0) < (minLen || 0);
    }
    public checkChipMaxLen(chip: string | null | undefined, maxLen: number | null | undefined): boolean {
        return (chip?.length || 0) > (maxLen || 0);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        return !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
    };

    private prepareFormGroup(
        isRequired: boolean | null, maxLen: number | null, minLen: number | null, maxAmount: number | null, minAmount: number | null,
    ): void {
        this.formControl.clearValidators();
        const newValidator: ValidatorFn[] = [
            ...(!!isRequired ? [this.requiredValidator] : []),
            ...((minLen || 0) > 0 ? [this.minLengthValidator] : []),
            ...((maxLen || 0) > 0 ? [this.maxLengthValidator] : []),
            ...((minAmount || 0) > 0 ? [this.minAmountValidator] : []),
            ...((maxAmount || 0) > 0 ? [this.maxAmountValidator] : []),
        ];
        this.formControl.setValidators([...newValidator, this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
    private requiredValidator: ValidatorFn = (): ValidationErrors | null => {
        const curr: string[] | null = this.formControl.value;
        const length = (curr || []).length;
        return !!curr && length == 0 ? { "required": true } : null;
    }
    private minAmountValidator: ValidatorFn = (): ValidationErrors | null => {
        const curr: string[] | null = this.formControl.value;
        const length = (curr || []).length;
        const minAmnt = this.minAmount || 0;
        return !!curr && length > 0 && length < minAmnt ? { "minAmount": { "actualAmount": length, "requiredAmount": minAmnt } } : null;
    }
    private maxAmountValidator: ValidatorFn = (): ValidationErrors | null => {
        const curr: string[] | null = this.formControl.value;
        const length = (curr || []).length;
        const maxAmnt = this.maxAmount || 0;
        return !!curr && length > 0 && length > maxAmnt ? { "maxAmount": { "actualAmount": length, "requiredAmount": maxAmnt } } : null;
    }
    private minLengthValidator: ValidatorFn = (): ValidationErrors | null => {
        let result: ValidationErrors | null = null;
        const minLen = this.minLen || 0;
        const currValuesList: string[] = this.formControl.value || [];
        for (let index = 0; index < currValuesList.length && result == null && minLen > 0; index++) {
            const item = currValuesList[index];
            if (item.length < minLen) {
                result = { "minlength": { "requiredLength": minLen, "actualLength": item.length, "actualValue": item } };
            }
        }
        return result;
    }
    private maxLengthValidator: ValidatorFn = (): ValidationErrors | null => {
        let result: ValidationErrors | null = null;
        const maxLen = this.maxLen || 0;
        const currValuesList: string[] = this.formControl.value || [];
        for (let index = 0; index < currValuesList.length && result == null && maxLen > 0; index++) {
            const item = currValuesList[index];
            if (item.length > maxLen) {
                result = { "maxlength": { "requiredLength": maxLen, "actualLength": item.length, "actualValue": item } };
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

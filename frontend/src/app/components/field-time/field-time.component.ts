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
import { TranslatePipe } from '@ngx-translate/core';

import { TimeUtil } from 'src/app/utils/time.util';

export const FT_DEFAULT_STEP = 60;
export const FT_LENGTH_MIN = 5;
export const FT_TIME_REGEX = '^([01][0-9]|2[0-3]):[0-5][0-9]$';

@Component({
    selector: 'app-field-time',
    exportAs: 'appFieldTime',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, TranslatePipe],
    templateUrl: './field-time.component.html',
    styleUrl: './field-time.component.scss',
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
    public min: string | null = null;  // Valid values: ^([01][0-9]|2[0-3]):[0-5][0-9]$
    @Input()
    public max: string | null = null;  // Valid values: ^([01][0-9]|2[0-3]):[0-5][0-9]$
    @Input()
    public step: number = FT_DEFAULT_STEP;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ time: this.formControl });
    public errMessage: string = '';
    public innMin: string | null = null;
    public innMax: string | null = null;
    public innMinLimit: string | null = null;
    public innMaxLimit: string | null = null;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired'] || !!changes['min'] || !!changes['max']) {
            const { min, minLimit } = this.getValueMin(this.min);
            this.innMin = min;
            this.innMinLimit = minLimit;
            const { max, maxLimit } = this.getValueMax(this.max);
            this.innMax = max;
            this.innMaxLimit = maxLimit;
            this.prepareFormGroup(this.isRequired, this.innMin, this.innMax);
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(this.isDisabled);
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
            result = !result && 'required' === error ? 'ExpectationFailed.time:required' : result;
            result = !result && 'min' === error ? 'ExpectationFailed.time:min' : result;
            result = !result && 'max' === error ? 'ExpectationFailed.time:max' : result;
        }
        return result;
    }

    // ** Private API **

    private getValueMin(min: string | null): { min: string | null, minLimit: string | null } {
        let resMin: string | null = null;
        let resMinLimit: string | null = null;
        if (!!min && (new RegExp(FT_TIME_REGEX)).test(min)) {
            resMin = min;
            resMinLimit = min;
            const dataMin = TimeUtil.parseTime(min);
            if (!!dataMin && (dataMin.hours > 0 || dataMin.minutes > 0)) {
                const limit = TimeUtil.addTime(min, 0, -1, 0);
                if (!!limit) {
                    resMinLimit = ('00' + limit.hours).slice(-2) + ':' + ('00' + limit.minutes).slice(-2);
                }
            }
        }
        return { min: resMin, minLimit: resMinLimit };
    }
    private getValueMax(max: string | null): { max: string | null, maxLimit: string | null } {
        let resMax: string | null = null;
        let resMaxLimit: string | null = null;
        if (!!max && (new RegExp(FT_TIME_REGEX)).test(max)) {
            resMax = max;
            resMaxLimit = max;
            const dataMax = TimeUtil.parseTime(max);
            if (!!dataMax && (dataMax.hours < 23 || dataMax.minutes < 59)) {
                const limit = TimeUtil.addTime(max, 0, +1, 0);
                if (!!limit) {
                    resMaxLimit = ('00' + limit.hours).slice(-2) + ':' + ('00' + limit.minutes).slice(-2);
                }
            }
        }
        return { max: resMax, maxLimit: resMaxLimit };
    }
    private prepareFormGroup(isRequired: boolean, min: string | null, max: string | null): void {
        this.formControl.clearValidators();
        const newValidator: ValidatorFn[] = [
            ...(isRequired ? [Validators.required] : []),
            ...((min || '').length >= FT_LENGTH_MIN ? [this.timeMinValidator] : []),
            ...((max || '').length >= FT_LENGTH_MIN ? [this.timeMaxValidator] : []),
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
            ? { 'min': { 'requiredMin': this.min, 'actual': this.formControl.value } }
            : null;
    }
    private timeMaxValidator: ValidatorFn = (): ValidationErrors | null => {
        const curr = TimeUtil.parseTime(this.formControl.value || '');
        const max = TimeUtil.parseTime(this.max || '');
        return !!curr && !!max && this.getSeconds(max) < this.getSeconds(curr)
            ? { 'max': { 'requiredMax': this.max, 'actual': this.formControl.value } }
            : null;
    }
}

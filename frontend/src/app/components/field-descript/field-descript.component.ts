import {
  ChangeDetectionStrategy,
  Component,
  Input,
  OnChanges,
  SimpleChanges,
  ViewChild,
  ViewEncapsulation,
  forwardRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  AbstractControl,
  ControlValueAccessor,
  FormControl,
  FormGroup,
  NG_VALIDATORS,
  NG_VALUE_ACCESSOR,
  ReactiveFormsModule,
  ValidationErrors,
  Validator,
  ValidatorFn,
  Validators,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslateModule } from '@ngx-translate/core';
  
export const DESCRIPT_MIN_LENGTH = 5;
export const DESCRIPT_MAX_LENGTH = 2000;
export const DESCRIPT_MIN_ROWS = 10;
export const DESCRIPT_MAX_ROWS = 10;

@Component({
  selector: 'app-field-descript',
  standalone: true,
  imports: [CommonModule, TranslateModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule],
  templateUrl: './field-descript.component.html',
  styleUrls: ['./field-descript.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldDescriptComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldDescriptComponent), multi: true },
  ],
})
export class FieldDescriptComponent  implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public isReadOnly: boolean = false;
  @Input()
  public isRequired: boolean = false;
  @Input()
  public label: string = 'field-descript.label';
  @Input()
  public minLen = DESCRIPT_MIN_LENGTH;
  @Input()
  public maxLen = DESCRIPT_MAX_LENGTH;
  @Input()
  public isDisabled = false;
  @Input()
  public hint: string = '';
  @Input()
  public minRows = DESCRIPT_MIN_ROWS;
  @Input()
  public maxRows = DESCRIPT_MAX_ROWS;
  @ViewChild(MatInput, { static: false })
  public matInput: MatInput | null = null;

  public formControl: FormControl = new FormControl({value: null, disabled: false}, []);
  public formGroup: FormGroup = new FormGroup({ description: this.formControl });

  constructor() {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen']) {
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
    this.matInput?.focus();
  }

  public getErrorMsg(errors: ValidationErrors | null): string {
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? 'Validation.descript:required' : result;
      result = !result && 'minlength' === error ? 'Validation.descript:min_length' : result;
      result = !result && 'maxlength' === error ? 'Validation.descript:max_length' : result;
    }
    return result;
  }

  // ** Private API **

  private prepareFormGroup(): void {
    this.formControl.clearValidators();
    const newValidator: ValidatorFn[] = [
      ...(this.isRequired ? [Validators.required] : []),
      ...(this.minLen > 0 ? [Validators.minLength(this.minLen)] : []),
      ...(this.maxLen > 0 ? [Validators.maxLength(this.maxLen)] : []),
    ];
    this.formControl.setValidators(newValidator);
  }
}

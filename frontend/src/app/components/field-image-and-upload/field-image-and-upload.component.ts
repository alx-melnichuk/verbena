import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation,
  forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FieldFileUploadComponent } from '../field-file-upload/field-file-upload.component';
import {
  AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, ValidationErrors,
  Validator
} from '@angular/forms';

@Component({
  selector: 'app-field-image-and-upload',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, FieldFileUploadComponent],
  templateUrl: './field-image-and-upload.component.html',
  styleUrls: ['./field-image-and-upload.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldImageAndUploadComponent), multi: true },
    { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldImageAndUploadComponent), multi: true },
  ],
})
export class FieldImageAndUploadComponent implements OnChanges, ControlValueAccessor, Validator {
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public isReadonly: boolean = false;
  @Input()
  public maxSize = -1;
  @Input()
  public validTypes = '';
  
  @Output()
  readonly addFile: EventEmitter<File> = new EventEmitter();
  @Output()
  readonly readFile: EventEmitter<string[]> = new EventEmitter();
  @Output()
  readonly deleteFile: EventEmitter<void> = new EventEmitter();
  
  @HostBinding('class.is-disabled')
  public isDisabledVal: boolean = false;
  @HostBinding('class.is-non-event')
  public get isNonEvent(): boolean {
    return this.isDisabledVal || this.isReadonly;
  }

  public imageFile: File | null | undefined;
  public imageView: string = '';
  public initIsImage: boolean | undefined;
  
  public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
  public formGroup: FormGroup = new FormGroup({ image: this.formControl });

  constructor(
    private changeDetectorRef: ChangeDetectorRef,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isDisabled']) {
      this.setDisabledState(this.isDisabled);
    }
  }

  // ** ControlValueAccessor - start **

  public onChange: (val: string | null) => void = () => {};
  public onTouched: () => void = () => {};

  public writeValue(value: any): void {
    this.imageView = value || '';
    this.formControl.setValue(value, { emitEvent: true });
    if (this.initIsImage === undefined) {
      this.initIsImage = !!value;
    }
  }

  public registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  public registerOnTouched(fn: any): void {
    this.onTouched = fn;
  }

  public setDisabledState(isDisabled: boolean): void {
    if (this.isDisabledVal != isDisabled) {
      this.isDisabledVal = isDisabled;
      isDisabled ? this.formGroup.disable() : this.formGroup.enable();
    }
  }

  // ** ControlValueAccessor - finish **

  // ** Validator - start **

  public validate(control: AbstractControl): ValidationErrors | null {
    return this.formControl.errors;
  }

  // ** Validator - finish **

  // ** Public API **
  
  public addImage(file: File): void {
    if (this.isDisabledVal || this.isReadonly) {
      return;
    }
    this.imageFile = file;
    this.formControl.setValue(file.name, { emitEvent: true });
    this.onTouched();
    this.onChange(file.name);
    this.addFile.emit(file);
  }

  public readImage(buffFile: string[]): void {
    if (this.isDisabledVal || this.isReadonly) {
        return;
    }
    if (buffFile.length > 0) {
      this.imageView = buffFile[1];
      this.changeDetectorRef.markForCheck();
    }
    this.readFile.emit(buffFile);
  }

  public deleteImage(): void {
    if (this.isDisabledVal || this.isReadonly) {
      return;
    }
    this.imageFile = (!!this.initIsImage ? null : undefined);
    this.imageView = '';
    this.formControl.setValue(null, { emitEvent: true });
    this.onTouched();
    this.onChange(null);
    this.deleteFile.emit();
  }

  // ** Private API **

}

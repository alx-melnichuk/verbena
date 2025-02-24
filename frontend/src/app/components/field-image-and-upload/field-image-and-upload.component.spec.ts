import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldImageAndUploadComponent } from './field-image-and-upload.component';

describe('FieldImageAndUploadComponent', () => {
  let component: FieldImageAndUploadComponent;
  let fixture: ComponentFixture<FieldImageAndUploadComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldImageAndUploadComponent]
    });
    fixture = TestBed.createComponent(FieldImageAndUploadComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

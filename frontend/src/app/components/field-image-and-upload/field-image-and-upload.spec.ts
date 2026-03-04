import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldImageAndUpload } from './field-image-and-upload';

describe('FieldImageAndUpload', () => {
  let component: FieldImageAndUpload;
  let fixture: ComponentFixture<FieldImageAndUpload>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldImageAndUpload]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldImageAndUpload);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

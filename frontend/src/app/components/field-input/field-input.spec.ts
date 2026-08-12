import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldInput } from './field-input';

describe('FieldInput', () => {
  let component: FieldInput;
  let fixture: ComponentFixture<FieldInput>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldInput]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldInput);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

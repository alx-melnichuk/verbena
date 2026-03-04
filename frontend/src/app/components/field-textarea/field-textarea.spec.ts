import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldTextarea } from './field-textarea';

describe('FieldTextarea', () => {
  let component: FieldTextarea;
  let fixture: ComponentFixture<FieldTextarea>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldTextarea]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldTextarea);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

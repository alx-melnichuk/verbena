import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldChipGridComponent } from './field-chip-grid.component';

describe('FieldChipGridComponent', () => {
  let component: FieldChipGridComponent;
  let fixture: ComponentFixture<FieldChipGridComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldChipGridComponent]
    });
    fixture = TestBed.createComponent(FieldChipGridComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

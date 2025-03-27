import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgStreamListComponent } from './pg-stream-list.component';

describe('PgStreamListComponent', () => {
  let component: PgStreamListComponent;
  let fixture: ComponentFixture<PgStreamListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgStreamListComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgStreamListComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

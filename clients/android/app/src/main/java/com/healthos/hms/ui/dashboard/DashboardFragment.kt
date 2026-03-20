package com.healthos.hms.ui.dashboard

import android.os.Bundle
import android.view.*
import androidx.fragment.app.Fragment
import androidx.fragment.app.viewModels
import androidx.lifecycle.lifecycleScope
import androidx.recyclerview.widget.LinearLayoutManager
import com.healthos.hms.R
import com.healthos.hms.adapter.PatientAdapter
import com.healthos.hms.adapter.AppointmentAdapter
import com.healthos.hms.databinding.FragmentDashboardBinding
import com.healthos.hms.ui.viewmodel.DashboardViewModel
import com.healthos.hms.utils.formatCurrency
import com.healthos.hms.utils.formatDate
import kotlinx.coroutines.launch

class DashboardFragment : Fragment() {
    
    private var _binding: FragmentDashboardBinding? = null
    private val binding get() = _binding!!
    
    private val viewModel: DashboardViewModel by viewModels()
    private lateinit var patientAdapter: PatientAdapter
    private lateinit var appointmentAdapter: AppointmentAdapter
    
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        _binding = FragmentDashboardBinding.inflate(inflater, container, false)
        return binding.root
    }
    
    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        
        setupRecyclerViews()
        setupObservers()
        setupClickListeners()
        
        // Load initial data
        viewModel.loadDashboardData()
    }
    
    private fun setupRecyclerViews() {
        // Recent patients RecyclerView
        patientAdapter = PatientAdapter { patient ->
            // Navigate to patient details
            viewModel.navigateToPatientDetails(patient.id)
        }
        
        binding.recyclerViewRecentPatients.apply {
            layoutManager = LinearLayoutManager(context)
            adapter = patientAdapter
        }
        
        // Today's appointments RecyclerView
        appointmentAdapter = AppointmentAdapter { appointment ->
            // Navigate to appointment details
            viewModel.navigateToAppointmentDetails(appointment.id)
        }
        
        binding.recyclerViewTodayAppointments.apply {
            layoutManager = LinearLayoutManager(context)
            adapter = appointmentAdapter
        }
    }
    
    private fun setupObservers() {
        viewLifecycleOwner.lifecycleScope.launch {
            viewModel.dashboardStats.collect { stats ->
                stats?.let {
                    binding.textViewTotalPatients.text = it.totalPatients.toString()
                    binding.textViewTodayAppointments.text = it.todayAppointments.toString()
                    binding.textViewPendingInvoices.text = it.pendingInvoices.toString()
                    binding.textViewMonthlyRevenue.text = formatCurrency(it.monthlyRevenue)
                    binding.textViewOccupancyRate.text = "${(it.occupiedBeds * 100 / it.totalBeds)}%"
                }
            }
        }
        
        viewLifecycleOwner.lifecycleScope.launch {
            viewModel.recentPatients.collect { patients ->
                patientAdapter.submitList(patients)
                binding.textViewNoPatients.visibility = if (patients.isEmpty()) View.VISIBLE else View.GONE
            }
        }
        
        viewLifecycleOwner.lifecycleScope.launch {
            viewModel.todayAppointments.collect { appointments ->
                appointmentAdapter.submitList(appointments)
                binding.textViewNoAppointments.visibility = if (appointments.isEmpty()) View.VISIBLE else View.GONE
            }
        }
        
        viewLifecycleOwner.lifecycleScope.launch {
            viewModel.isLoading.collect { isLoading ->
                binding.progressBar.visibility = if (isLoading) View.VISIBLE else View.GONE
                binding.scrollViewContent.visibility = if (isLoading) View.GONE else View.VISIBLE
            }
        }
        
        viewLifecycleOwner.lifecycleScope.launch {
            viewModel.error.collect { error ->
                error?.let {
                    // Show error message
                    binding.textViewError.text = it
                    binding.textViewError.visibility = View.VISIBLE
                } ?: run {
                    binding.textViewError.visibility = View.GONE
                }
            }
        }
    }
    
    private fun setupClickListeners() {
        // Refresh button
        binding.buttonRefresh.setOnClickListener {
            viewModel.loadDashboardData()
        }
        
        // Quick action buttons
        binding.buttonNewPatient.setOnClickListener {
            viewModel.navigateToNewPatient()
        }
        
        binding.buttonNewAppointment.setOnClickListener {
            viewModel.navigateToNewAppointment()
        }
        
        binding.buttonNewInvoice.setOnClickListener {
            viewModel.navigateToNewInvoice()
        }
        
        binding.buttonViewAllPatients.setOnClickListener {
            viewModel.navigateToPatientsList()
        }
        
        binding.buttonViewAllAppointments.setOnClickListener {
            viewModel.navigateToAppointmentsList()
        }
        
        binding.buttonViewAllInvoices.setOnClickListener {
            viewModel.navigateToInvoicesList()
        }
        
        // Stats cards click listeners
        binding.cardTotalPatients.setOnClickListener {
            viewModel.navigateToPatientsList()
        }
        
        binding.cardTodayAppointments.setOnClickListener {
            viewModel.navigateToAppointmentsList()
        }
        
        binding.cardPendingInvoices.setOnClickListener {
            viewModel.navigateToInvoicesList()
        }
        
        binding.cardMonthlyRevenue.setOnClickListener {
            viewModel.navigateToFinancialReports()
        }
    }
    
    override fun onCreateOptionsMenu(menu: Menu, inflater: MenuInflater) {
        inflater.inflate(R.menu.dashboard_menu, menu)
        return true
    }
    
    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            R.id.action_refresh -> {
                viewModel.loadDashboardData()
                true
            }
            R.id.action_settings -> {
                viewModel.navigateToSettings()
                true
            }
            R.id.action_logout -> {
                viewModel.logout()
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }
    
    override fun onResume() {
        super.onResume()
        // Refresh data when fragment becomes visible
        viewModel.loadDashboardData()
    }
    
    override fun onDestroyView() {
        super.onDestroyView()
        _binding = null
    }
}
